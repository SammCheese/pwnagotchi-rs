use std::{borrow::Cow, sync::Arc};

use anyhow::Result;
use axum::{
  Router,
  body::Body,
  extract::{FromRequestParts, Path},
  http::{Request, StatusCode, header},
  middleware::{self, Next},
  response::{IntoResponse, Response},
  routing::get,
};
use axum_auth::AuthBasic;
use include_dir::{Dir, include_dir};
use parking_lot::RwLock;
use pwnagotchi_shared::{
  config::config,
  identity::Identity,
  logger::LOGGER,
  sessions::manager::SessionManager,
  traits::{
    general::{Component, CoreModules, Dependencies},
    ui::ServerTrait,
  },
};
use tokio::{sync::oneshot, task::JoinHandle};

use crate::web::pages::handler::{
  inbox_handler, index_handler, message_handler, new_message_handler, peers_handler,
  plugins_handler, profile_handler, status_handler, ui,
};

pub static TEMPLATE_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/templates");
pub static STATIC_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/static");
pub static FONT_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/fonts");

pub struct ServerComponent {
  server: Option<Arc<dyn ServerTrait + Send + Sync>>,
}

impl Dependencies for ServerComponent {
  fn name(&self) -> &'static str {
    "WebUIComponent"
  }

  fn dependencies(&self) -> &[&str] {
    &["SessionManager", "Identity"]
  }
}

#[async_trait::async_trait]
impl Component for ServerComponent {
  async fn init(&mut self, ctx: &CoreModules) -> Result<()> {
    let (sm, identity) = (&ctx.session_manager, &ctx.identity);
    let sm = Arc::clone(sm);
    let identity = Arc::clone(identity);
    self.server = Some(Arc::new(Server::new(sm, identity)));
    Ok(())
  }

  async fn start(&self) -> Result<Option<JoinHandle<()>>> {
    if let Some(server) = &self.server {
      let server = Arc::clone(server);
      let handle = tokio::spawn(async move {
        server.start_server().await.unwrap_or_else(|e| {
          LOGGER.log_error("Server", &format!("Failed to start server: {e}"));
        });
      });
      return Ok(Some(handle));
    }
    Ok(None)
  }
}

impl Dependencies for Server {
  fn name(&self) -> &'static str {
    "WebUI"
  }

  fn dependencies(&self) -> &[&str] {
    &["SessionManager", "Identity"]
  }
}

impl Default for ServerComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl ServerComponent {
  #[must_use]
  pub fn new() -> Self {
    Self { server: None }
  }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct WebUIState {
  pub sm: Arc<SessionManager>,
  pub identity: Arc<RwLock<Identity>>,
}

pub struct Server {
  pub sm: Arc<SessionManager>,
  pub identity: Arc<RwLock<Identity>>,
  pub address: Cow<'static, str>,
  pub port: u16,
  #[allow(dead_code)]
  shutdown_tx: Option<oneshot::Sender<()>>,
}

#[async_trait::async_trait]
impl ServerTrait for Server {
  async fn start_server(&self) -> Result<(), String> {
    if self.address.is_empty() {
      LOGGER.log_info("Server", "Couldn't get IP of USB0, video server not starting");
      return Err("Couldn't get IP of USB0, video server not starting".into());
    }

    let sm = Arc::clone(&self.sm);
    let identity = Arc::clone(&self.identity);

    let app = build_router(sm, identity);

    let addr = format!("{}:{}", self.address, self.port);

    tokio::spawn(async move {
      match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => {
          if let Err(e) = axum::serve(listener, app).await {
            LOGGER.log_error("Server", &format!("Failed to serve: {e}"));
          }
        }
        Err(e) => {
          LOGGER.log_error("Server", &format!("Failed to bind to {addr}: {e}"));
        }
      }
    });
    Ok(())
  }

  async fn stop_server(&self) {}
}

impl Server {
  #[must_use]
  pub fn new(sm: Arc<SessionManager>, identity: Arc<RwLock<Identity>>) -> Self {
    let cfg = &config().ui.web;
    Self {
      sm,
      identity,
      address: cfg.address.clone(),
      port: cfg.port,
      shutdown_tx: None,
    }
  }
}

async fn basic_auth_middleware(req: Request<Body>, next: Next) -> impl IntoResponse {
  let cfg = &config().ui.web;

  if cfg.username.is_empty() || cfg.password.is_empty() {
    return next.run(req).await;
  }

  let (mut parts, body) = req.into_parts();
  let auth = AuthBasic::from_request_parts(&mut parts, &()).await;

  if let Ok(AuthBasic((user, password))) = auth {
    let pass = password.unwrap_or_default();
    if compare_safely(&user, cfg.username.as_ref()) && compare_safely(&pass, cfg.password.as_ref())
    {
      let req = Request::from_parts(parts, body);
      return next.run(req).await;
    }
  }

  (
    StatusCode::UNAUTHORIZED,
    [(axum::http::header::WWW_AUTHENTICATE, r#"Basic realm="User""#)],
    "Unauthorized",
  )
    .into_response()
}

fn compare_safely(a: &str, b: &str) -> bool {
  if a.len() != b.len() {
    return false;
  }
  let diff = a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y));
  diff == 0
}

pub fn build_router(sm: Arc<SessionManager>, identity: Arc<RwLock<Identity>>) -> Router {
  let state = Arc::new(WebUIState { sm, identity });
  Router::new()
    .layer(middleware::from_fn(basic_auth_middleware))
    // Template routes
    .route("/", get(index_handler))
    .route("/index", get(index_handler))
    .route("/ui", get(ui))
    // System Actions
    .route("/shutdown", get(static_handler))
    .route("/reboot", get(static_handler))
    .route("/restart", get(static_handler))
    // Inbox
    .route("/inbox", get(inbox_handler))
    .route("/inbox/profile", get(profile_handler))
    .route("/inbox/peers", get(peers_handler))
    .route("/inbox/{id}", get(peers_handler))
    .route("/inbox/{id}/mark", get(peers_handler))
    .route("/inbox/new", get(new_message_handler))
    .route("/inbox/send", get(new_message_handler))
    // Plugins
    .route("/plugins", get(plugins_handler))
    .route("/status", get(status_handler))
    .route("/message", get(message_handler))
    // Static
    .route("/{*path}", get(static_handler))
    .with_state(state)
}

pub async fn static_handler(Path(path): Path<String>) -> Response {
  let path = if path.is_empty() { "index.html" } else { path.as_str() };

  STATIC_ASSETS.get_file(path).map_or_else(
    || (StatusCode::NOT_FOUND, format!("File not found: {path}")).into_response(),
    |file| {
      let mime = mime_guess::from_path(path).first_or_octet_stream();

      Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(file.contents().into())
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed").into_response())
    },
  )
}

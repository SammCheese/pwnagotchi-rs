use std::{borrow::Cow, sync::Arc};

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
use pwnagotchi_shared::{
  config::config, identity::Identity, logger::LOGGER, sessions::manager::SessionManager,
};
use tokio::sync::oneshot;

use crate::web::pages::handler::{
  inbox_handler, index_handler, message_handler, new_message_handler, peers_handler,
  plugins_handler, profile_handler, status_handler, ui,
};

pub static TEMPLATE_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/templates");
pub static STATIC_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/static");
pub static FONT_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/fonts");

pub struct Server {
  pub address: Cow<'static, str>,
  pub port: u16,
  shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Default for Server {
  fn default() -> Self {
    Self::new()
  }
}

impl Server {
  #[must_use]
  pub fn new() -> Self {
    let cfg = &config().ui.web;
    Self {
      address: cfg.address.clone(),
      port: cfg.port,
      shutdown_tx: None,
    }
  }
  /// Starts the server.
  ///
  /// # Panics
  ///
  /// This function will panic if it fails to bind to address.
  pub fn start(&mut self, sm: &Arc<SessionManager>, identity: &Arc<Identity>) {
    if self.address.is_empty() {
      LOGGER.log_info("Server", "Couldn't get IP of USB0, video server not starting");
    } else {
      let addr = format!("{}:{}", self.address, self.port);

      let (shutdown_tx, shutdown_rx) = oneshot::channel();
      self.shutdown_tx = Some(shutdown_tx);

      let sm = Arc::clone(sm);
      let identity = Arc::clone(identity);

      tokio::spawn(async move {
        let app = build_router(sm, identity);

        match tokio::net::TcpListener::bind(&addr).await {
          Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
              LOGGER.log_error("Server", &format!("Failed to serve: {e}"));
              shutdown_rx.await.ok();
            }
          }
          Err(e) => {
            LOGGER.log_error("Server", &format!("Failed to bind to {addr}: {e}"));
          }
        }
      });
    }
  }

  pub fn stop(&mut self) {
    if let Some(tx) = self.shutdown_tx.take() {
      let _ = tx.send(());
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

pub fn build_router(sm: Arc<SessionManager>, identity: Arc<Identity>) -> Router {
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
    .with_state((sm, identity))
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

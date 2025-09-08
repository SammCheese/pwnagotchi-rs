use std::borrow::Cow;

use axum::{
  Router,
  extract::Path,
  http::{StatusCode, header},
  response::{IntoResponse, Response},
  routing::get,
};
use include_dir::{Dir, include_dir};
use pwnagotchi_shared::{config::config, log::LOGGER};

use crate::web::routes::{
  inbox_handler, index_handler, message_handler, new_message_handler, peers_handler,
  plugins_handler, profile_handler, status_handler, ui,
};

pub static TEMPLATE_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/templates");
pub static STATIC_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/static");
pub static FONT_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/fonts");

pub struct Server {
  pub address: Cow<'static, str>,
  pub port: u16,
}

impl Default for Server {
  fn default() -> Self {
    let address = &config().ui.web.address;
    let port = config().ui.web.port;
    Self { address: Cow::Borrowed(address), port }
  }
}

impl Server {
  #[must_use]
  pub fn new() -> Self {
    Self {
      address: Cow::Borrowed(&config().ui.web.address),
      port: config().ui.web.port,
    }
  }

  pub fn start(&self) {
    if self.address.is_empty() {
      LOGGER.log_info("Server", "Couldn't get IP of USB0, video server not starting");
    } else {
      let addr = format!("{}:{}", self.address, self.port);

      tokio::spawn(async move {
        let app = build_router();

        match tokio::net::TcpListener::bind(&addr).await {
          Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
              LOGGER.log_error("Server", &format!("Failed to Server: {e}"));
            }
          }
          Err(e) => {
            LOGGER.log_error("Server", &format!("Failed to bind to {addr}: {e}"));
          }
        }
      });
    }
  }

  pub const fn stop(&self) {
    // TODO: Stop the server
  }
}

pub fn build_router() -> Router {
  Router::new()
    // Template routes
    .route("/", get(index_handler))
    .route("/index", get(index_handler))
    .route("/inbox", get(inbox_handler))
    .route("/inbox/new", get(new_message_handler))
    .route("/inbox/peers", get(peers_handler))
    .route("/plugins", get(plugins_handler))
    .route("/inbox/profile", get(profile_handler))
    .route("/status", get(status_handler))
    .route("/message", get(message_handler))
    .route("/ui", get(ui))
    .route("/{*path}", get(static_handler))
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

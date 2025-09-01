use crate::core::{
    config::config,
    log::LOGGER,
    ui::old::web::routes::{inbox_handler, index_handler, ui},
};
use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use include_dir::{Dir, include_dir};
use std::sync::Arc;
use tera::Tera;

pub static TEMPLATE_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/templates");
pub static STATIC_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/static");

pub struct Server {
    pub address: String,
    pub port: u16,
}

impl Default for Server {
    fn default() -> Self {
        let address = config().ui.web.address.clone();
        let port = config().ui.web.port;
        Self { address, port }
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            address: config().ui.web.address.clone(),
            port: config().ui.web.port,
        }
    }

    pub fn start(&self) {
        if self.address.is_empty() {
            LOGGER.log_info(
                "Server",
                "Couldn't get IP of USB0, video server not starting",
            );
        } else {
            let addr = format!("{}:{}", self.address, self.port);

            tokio::spawn(async move {
                let tera = Arc::new(build_tera_from_include_dir());
                let app = build_router(tera);

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
        // Stop the server
    }
}

pub fn build_router(tera: Arc<Tera>) -> Router {
    Router::new()
        // Template routes
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/inbox.html", get(inbox_handler))
        .route("/ui", get(ui))
        .route("/{*path}", get(static_handler))
        // Tera extension
        .layer(axum::Extension(tera))
}

pub async fn static_handler(Path(path): Path<String>) -> Response {
    // allow /index.html etc.
    let path = if path.is_empty() {
        "index.html"
    } else {
        path.as_str()
    };

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

fn build_tera_from_include_dir() -> Tera {
    let mut templates = Vec::new();

    for file in TEMPLATE_ASSETS.files() {
        let Some(path) = file.path().to_str() else {
            eprintln!("Failed to convert path to str");
            continue;
        };
        if let Ok(contents) = std::str::from_utf8(file.contents()) {
            templates.push((path, contents.to_string()));
        } else {
            eprintln!("Failed to parse template file '{path}' as UTF-8");
        }
    }

    let mut tera = Tera::default();
    tera.add_raw_templates(templates).unwrap_or_else(|e| {
        eprintln!("Failed to add templates to Tera: {e}");
    });
    tera
}

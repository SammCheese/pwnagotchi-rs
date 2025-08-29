#![allow(clippy::unwrap_used, clippy::missing_panics_doc)]

use std::{collections::HashMap, f64::consts::E, sync::Arc};

use serde::Serialize;
use tera::{Tera, Context};
use axum::{extract::Path, http::{header, StatusCode}, response::{Html, IntoResponse, Response}, routing::get, Extension, Router};
use crate::core::{config::config, ui::old::web::{frame::{FRAME_LOCK, FRAME_PATH}}};

#[derive(Serialize)]
struct NavItem {
    url: String,
    id: String,
    icon: String,
    label: String,
}

fn default_navigation() -> Vec<NavItem> {
    vec![
        NavItem { url: "/".to_string(), id: "home".to_string(), icon: "eye".to_string(), label: "Home".to_string() },
        NavItem { url: "/inbox".to_string(), id: "inbox".to_string(), icon: "bars".to_string(), label: "Inbox".to_string() },
        NavItem { url: "/inbox/new".to_string(), id: "new".to_string(), icon: "mail".to_string(), label: "New".to_string() },
        NavItem { url: "/inbox/profile".to_string(), id: "profile".to_string(), icon: "info".to_string(), label: "Profile".to_string() },
        NavItem { url: "/inbox/peers".to_string(), id: "peers".to_string(), icon: "user".to_string(), label: "Peers".to_string() },
        NavItem { url: "/plugins".to_string(), id: "plugins".to_string(), icon: "grid".to_string(), label: "Plugins".to_string() },
    ]
}

pub async fn index_handler(Extension(tera): Extension<Arc<Tera>>) -> Html<String> {
    let mut ctx = Context::new();
    ctx.insert("title", config().main.name.as_str());
    ctx.insert("other_mode", "AUTO");
    ctx.insert("fingerprint", "XXXX");
    ctx.insert("navigations", &default_navigation());
    ctx.insert("active_page", "home");
    let rendered = tera.render("index.html", &ctx).unwrap();
    Html(rendered)
}

pub async fn inbox_handler(Extension(tera): Extension<Arc<Tera>>) -> Html<String> {
    let mut ctx = Context::new();
    use serde_json::json;
    ctx.insert("inbox", &json!({
      "pages": 0,
      "messages": [],
    }));
    let rendered = tera.render("inbox.html", &ctx).unwrap();
    Html(rendered)
}

pub async fn ui() -> impl IntoResponse {
  let handle = FRAME_LOCK.lock();
  let frame = match std::fs::read(FRAME_PATH) {
      Ok(data) => data,
      Err(e) => {
          return StatusCode::NOT_FOUND.into_response();
      }
  };
  drop(handle);

  Response::builder()
    .header(header::CONTENT_TYPE, "image/png")
    .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
    .header(header::PRAGMA, "no-cache")
    .header(header::EXPIRES, "0")
    .body(frame.into())
    .unwrap_or_else(|_|
      (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response").into_response()
    )
}

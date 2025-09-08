#![allow(clippy::missing_panics_doc, dead_code)]

use askama::Template;
use axum::{
  http::{StatusCode, header},
  response::{Html, IntoResponse, Response},
};
use pwnagotchi_shared::{config::config, mesh::peer::Peer};
use serde_json::Value;

use crate::web::frame::FRAME_PATH;

#[derive(Clone, Copy)]
struct NavItem {
  url: &'static str,
  id: &'static str,
  icon: &'static str,
  label: &'static str,
}

#[derive(Clone, Default)]
struct InboxCtx {
  pages: u32,
  messages: Vec<Message>,
}

#[derive(Clone, Default)]
struct Message {
  id: String,
  sender_name: String,
  sender: String,
  created_at: String,
  seen_at: Option<String>,
  seen: bool,
  data: Option<String>,
}

#[derive(Clone, Default)]
struct PluginCtx {
  name: String,
  description: Option<String>,
  has_webhook: bool,
  enabled: bool,
}

fn default_navigation() -> Vec<NavItem> {
  vec![
    NavItem {
      url: "/",
      id: "home",
      icon: "eye",
      label: "Home",
    },
    NavItem {
      url: "/inbox",
      id: "inbox",
      icon: "bars",
      label: "Inbox",
    },
    NavItem {
      url: "/inbox/new",
      id: "new",
      icon: "mail",
      label: "New",
    },
    NavItem {
      url: "/inbox/profile",
      id: "profile",
      icon: "info",
      label: "Profile",
    },
    NavItem {
      url: "/inbox/peers",
      id: "peers",
      icon: "user",
      label: "Peers",
    },
    NavItem {
      url: "/plugins",
      id: "plugins",
      icon: "grid",
      label: "Plugins",
    },
  ]
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
  title: String,
  other_mode: String,
  fingerprint: String,
  navigations: Vec<NavItem>,
  active_page: String,
  error: String,
}

#[derive(Template)]
#[template(path = "inbox.html")]
struct InboxTemplate {
  title: String,
  navigations: Vec<NavItem>,
  active_page: String,
  error: String,
  page: u32,
  inbox: InboxCtx,
}

#[derive(Template)]
#[template(path = "new_message.html")]
struct NewMessageTemplate {
  title: String,
  navigations: Vec<NavItem>,
  active_page: String,
  error: String,
  to: Option<String>,
}

#[derive(Template)]
#[template(path = "peers.html")]
struct PeersTemplate {
  title: String,
  navigations: Vec<NavItem>,
  active_page: String,
  error: String,
  name: String,
  peers: Vec<Peer>,
}

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfileTemplate {
  title: String,
  navigations: Vec<NavItem>,
  active_page: String,
  error: String,
  name: String,
  fingerprint: String,
  data: Value,
}

#[derive(Template)]
#[template(path = "plugins.html")]
struct PluginsTemplate {
  title: String,
  navigations: Vec<NavItem>,
  active_page: String,
  error: String,
  plugins: Vec<PluginCtx>,
  csrf_token: String,
}

#[derive(Template)]
#[template(path = "message.html")]
struct MessageTemplate {
  title: String,
  navigations: Vec<NavItem>,
  active_page: String,
  error: String,
  message: Message,
}

#[derive(Template)]
#[template(path = "status.html")]
struct StatusTemplate {
  title: String,
  navigations: Vec<NavItem>,
  active_page: String,
  error: String,
  message: String,
  go_back_after: u32,
}

pub async fn index_handler() -> impl IntoResponse {
  let tpl = IndexTemplate {
    title: config().main.name.to_string(),
    other_mode: "AUTO".to_string(),
    fingerprint: "XXXX".to_string(),
    navigations: default_navigation(),
    active_page: "home".to_string(),
    error: String::new(),
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn inbox_handler() -> impl IntoResponse {
  let tpl = InboxTemplate {
    title: config().main.name.to_string(),
    navigations: default_navigation(),
    active_page: "inbox".to_string(),
    error: String::new(),
    page: 1,
    inbox: InboxCtx { pages: 0, messages: vec![] },
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}
pub async fn new_message_handler() -> impl IntoResponse {
  let tpl = NewMessageTemplate {
    title: config().main.name.to_string(),
    navigations: default_navigation(),
    active_page: "new".to_string(),
    error: String::new(),
    to: None,
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn peers_handler() -> impl IntoResponse {
  let tpl = PeersTemplate {
    title: config().main.name.to_string(),
    navigations: default_navigation(),
    active_page: "peers".to_string(),
    error: String::new(),
    name: config().main.name.to_string(),
    peers: vec![],
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn profile_handler() -> impl IntoResponse {
  let tpl = ProfileTemplate {
    title: config().main.name.to_string(),
    navigations: default_navigation(),
    active_page: "profile".to_string(),
    error: String::new(),
    name: config().main.name.to_string(),
    fingerprint: "XXXX".to_string(),
    data: serde_json::json!({}),
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn plugins_handler() -> impl IntoResponse {
  let tpl = PluginsTemplate {
    title: "Plugins".to_string(),
    navigations: default_navigation(),
    active_page: "plugins".to_string(),
    error: String::new(),
    plugins: vec![],
    csrf_token: String::new(),
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn message_handler() -> impl IntoResponse {
  let tpl = MessageTemplate {
    title: "Message".to_string(),
    navigations: default_navigation(),
    active_page: "inbox".to_string(),
    error: String::new(),
    message: Message {
      id: String::new(),
      sender_name: String::new(),
      sender: String::new(),
      created_at: String::new(),
      seen_at: None,
      seen: false,
      data: None,
    },
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn status_handler() -> impl IntoResponse {
  let tpl = StatusTemplate {
    title: "Status".to_string(),
    navigations: default_navigation(),
    active_page: String::new(),
    error: String::new(),
    message: String::new(),
    go_back_after: 2,
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}
pub async fn ui() -> impl IntoResponse {
  let frame = match std::fs::read(FRAME_PATH.as_str()) {
    Ok(data) => data,
    Err(_e) => return StatusCode::NOT_FOUND.into_response(),
  };

  Response::builder()
    .header(header::CONTENT_TYPE, "image/png")
    .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
    .header(header::PRAGMA, "no-cache")
    .header(header::EXPIRES, "0")
    .body(frame.into())
    .unwrap_or_else(|_| {
      (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response").into_response()
    })
}

use std::sync::Arc;

use askama::Template;
use axum::{
  extract::State,
  http::{StatusCode, header},
  response::{Html, IntoResponse, Response},
};
use pwnagotchi_plugins::managers::plugin_manager::PluginState;
use pwnagotchi_shared::{config::config, models::agent::RunningMode};

use crate::web::{
  frame::FRAME_PATH,
  pages::routes::{
    BaseCtx, InboxCtx, InboxTemplate, IndexTemplate, Message, MessageTemplate, NavItem,
    NewMessageTemplate, PeersTemplate, PluginCtx, PluginsTemplate, ProfileTemplate, StatusTemplate,
  },
  server::WebUIState,
};

pub async fn index_handler() -> impl IntoResponse {
  let tpl = IndexTemplate {
    base: make_base("Home", "home"),
    other_mode: if true { RunningMode::Auto.to_string() } else { RunningMode::Manual.to_string() },
    fingerprint: "XXXX".to_string(),
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn inbox_handler() -> impl IntoResponse {
  let tpl = InboxTemplate {
    base: make_base("Inbox", "inbox"),
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
    base: make_base("New Message", "new"),
    to: None,
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn peers_handler() -> impl IntoResponse {
  let tpl = PeersTemplate {
    base: make_base("Peers", "peers"),
    name: config().main.name.to_string(),
    peers: vec![],
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn profile_handler(State(state): State<Arc<WebUIState>>) -> impl IntoResponse {
  let tpl = ProfileTemplate {
    base: make_base("Profile", "profile"),
    name: config().main.name.to_string(),
    fingerprint: state.identity.read().fingerprint().to_string(),
    data: serde_json::json!({}),
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn plugins_handler(State(state): State<Arc<WebUIState>>) -> impl IntoResponse {
  let handle = state.pluginmanager.write();
  let plugins = handle.get_plugins();

  let plugins: Vec<PluginCtx> = plugins
    .iter()
    .map(|p| PluginCtx {
      name: p.plugin.info().name.to_string(),
      description: Some(p.plugin.info().description.to_string()),
      version: p.plugin.info().version.to_string(),
      enabled: matches!(p.state, PluginState::Initialized),
      has_webhook: p.plugin.webhook().is_some(),
    })
    .collect();

  drop(handle);

  let tpl = PluginsTemplate {
    base: make_base("Plugins", "plugins"),
    plugins,
    csrf_token: String::new(),
  };
  match tpl.render() {
    Ok(s) => Html(s).into_response(),
    Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

pub async fn message_handler() -> impl IntoResponse {
  let tpl = MessageTemplate {
    base: make_base("Message", "inbox"),
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
    base: make_base("Status", "status"),
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

pub fn make_base(title: impl Into<String>, active_page: impl Into<String>) -> BaseCtx {
  BaseCtx {
    title: title.into(),
    navigations: default_navigation(),
    active_page: active_page.into(),
    error: String::new(),
  }
}

fn default_navigation() -> Vec<NavItem> {
  vec![
    NavItem {
      url: "/".to_string(),
      id: "home".to_string(),
      icon: "eye".to_string(),
      label: "Home".to_string(),
    },
    NavItem {
      url: "/inbox".to_string(),
      id: "inbox".to_string(),
      icon: "bars".to_string(),
      label: "Inbox".to_string(),
    },
    NavItem {
      url: "/inbox/new".to_string(),
      id: "new".to_string(),
      icon: "mail".to_string(),
      label: "New".to_string(),
    },
    NavItem {
      url: "/inbox/profile".to_string(),
      id: "profile".to_string(),
      icon: "info".to_string(),
      label: "Profile".to_string(),
    },
    NavItem {
      url: "/inbox/peers".to_string(),
      id: "peers".to_string(),
      icon: "user".to_string(),
      label: "Peers".to_string(),
    },
    NavItem {
      url: "/plugins".to_string(),
      id: "plugins".to_string(),
      icon: "grid".to_string(),
      label: "Plugins".to_string(),
    },
  ]
}

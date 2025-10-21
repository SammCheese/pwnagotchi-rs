#![allow(clippy::missing_panics_doc, dead_code)]

use askama::Template;
use pwnagotchi_shared::mesh::peer::Peer;
use serde_json::Value;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct NavItem {
  pub url: String,
  pub id: String,
  pub icon: String,
  pub label: String,
}

#[derive(Clone, Default)]
pub struct InboxCtx {
  pub pages: u32,
  pub messages: Vec<Message>,
}

#[derive(Clone, Default)]
pub struct Message {
  pub id: String,
  pub sender_name: String,
  pub sender: String,
  pub created_at: String,
  pub seen_at: Option<String>,
  pub seen: bool,
  pub data: Option<String>,
}

#[derive(Clone, Default)]
pub struct PluginCtx {
  pub name: String,
  pub version: String,
  pub description: Option<String>,
  pub has_webhook: bool,
  pub enabled: bool,
}

#[derive(Clone, Default)]
pub struct BaseCtx {
  pub title: String,
  pub navigations: Vec<NavItem>,
  pub active_page: String,
  pub error: String,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
  pub base: BaseCtx,
  pub other_mode: String,
  pub fingerprint: String,
}

#[derive(Template)]
#[template(path = "inbox.html")]
pub struct InboxTemplate {
  pub base: BaseCtx,
  pub page: u32,
  pub inbox: InboxCtx,
}

#[derive(Template)]
#[template(path = "new_message.html")]
pub struct NewMessageTemplate {
  pub base: BaseCtx,
  pub to: Option<String>,
}

#[derive(Template)]
#[template(path = "peers.html")]
pub struct PeersTemplate {
  pub base: BaseCtx,
  pub name: String,
  pub peers: Vec<Peer>,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
  pub base: BaseCtx,
  pub name: String,
  pub fingerprint: String,
  pub data: Value,
}

#[derive(Template)]
#[template(path = "plugins.html")]
pub struct PluginsTemplate {
  pub base: BaseCtx,
  pub plugins: Vec<PluginCtx>,
  pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "message.html")]
pub struct MessageTemplate {
  pub base: BaseCtx,
  pub message: Message,
}

#[derive(Template)]
#[template(path = "status.html")]
pub struct StatusTemplate {
  pub base: BaseCtx,
  pub message: String,
  pub go_back_after: u32,
}

use std::{
  collections::HashMap,
  panic::catch_unwind,
  sync::{Arc, Mutex, PoisonError},
  time::Duration,
};

use image::{Rgba, RgbaImage};

use crate::core::{
  automata::Automata,
  config::config,
  log::LOGGER,
  mesh::peer::Peer,
  models::net::{AccessPoint, Station},
  sessions::lastsession::LastSession,
  traits::agentobserver::AgentObserver,
  ui::{
    components::{LabeledValue, Line, TextStyle, TextWidget, Widget},
    fonts::STATUS_FONT_NAME,
    old::{hw::base::DisplayTrait, web::frame},
    state::{State, StateValue},
  },
  utils::{face_to_string, total_unique_handshakes},
  voice::Voice,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FaceType {
  LookR,
  LookL,
  LookRHappy,
  LookLHappy,
  Sleep,
  Sleep2,
  Awake,
  Bored,
  Intense,
  Cool,
  Happy,
  Grateful,
  Excited,
  Motivated,
  Demotivated,
  Smart,
  Lonely,
  Sad,
  Angry,
  Friend,
  Broken,
  Debug,
  Upload,
  Upload1,
  Upload2,
}

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);

pub struct View {
  pub voice: Arc<Mutex<Voice>>,
  pub state: Arc<Mutex<State>>,
  pub display: Arc<dyn DisplayTrait + Send + Sync>,
  pub inverted: bool,
  pub background_color: Rgba<u8>,
  pub foreground_color: Rgba<u8>,
  pub frozen: bool,
  pub ignore_changes: Arc<Vec<&'static str>>,
  pub render_callbacks: Arc<Vec<fn(&RgbaImage)>>,
  pub width: u32,
  pub height: u32,
}

impl View {
  pub fn new(display: &Arc<dyn DisplayTrait + Send + Sync>) -> Self {
    let inverted = config().ui.inverted;
    let background_color = if inverted { BLACK } else { WHITE };
    let foreground_color = if inverted { WHITE } else { BLACK };

    let layout = Arc::clone(display);

    let view = Self {
      display: Arc::clone(display),
      voice: Arc::new(Mutex::new(Voice::new())),
      state: Arc::new(Mutex::new(State::new())),
      inverted,
      background_color,
      foreground_color,
      frozen: false,
      ignore_changes: Arc::new(Vec::new()),
      render_callbacks: Arc::new(Vec::new()),
      width: layout.layout().width,
      height: layout.layout().height,
    };

    view.populate_state();
    view.configure_render_settings();

    view
  }

  pub fn configure_render_settings(&self) {
    if config().ui.fps < 1.0 {
      let mut cloned_ignore = Arc::clone(&self.ignore_changes);
      match Arc::get_mut(&mut cloned_ignore) {
        Some(ignore) => {
          ignore.push("uptime");
          ignore.push("name");
        }
        None => eprintln!("Failed to set ignored changes"),
      }

      LOGGER.log_warning("UI", "FPS set to 0, Display only updates for major changes");
    }
  }

  pub async fn start_render_loop(&self) {
    let delay = 1.0 / f64::from(config().ui.fps);
    loop {
      self.update(None, None);
      tokio::time::sleep(Duration::from_secs_f64(delay)).await;
    }
  }

  fn populate_state(&self) {
    let displ = Arc::clone(&self.display);
    let layout = displ.layout();
    let fontname = &STATUS_FONT_NAME.to_string();

    let channel = Self::make_channel_widget(layout.channel, fontname, self.foreground_color);
    let aps = Self::make_aps_widget(layout.aps, fontname, self.foreground_color);
    let uptime = Self::make_uptime_widget(layout.uptime, fontname, self.foreground_color);
    let line1 = Self::make_line_widget(layout.line1, self.foreground_color);
    let line2 = Self::make_line_widget(layout.line2, self.foreground_color);
    let face = Self::make_face_widget(layout.face, fontname, self.foreground_color);
    let friend_name =
      Self::make_friend_name_widget(layout.friend_name, fontname, self.foreground_color);
    let name = Self::make_name_widget(layout.name, fontname, self.foreground_color);
    let status = Self::make_status_widget(
      layout.status.pos,
      fontname,
      self.foreground_color,
      &self.voice.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    let shakes = Self::make_shakes_widget(layout.shakes, fontname, self.foreground_color);
    let mode = Self::make_mode_widget(layout.mode, fontname, self.foreground_color);

    self.add_element("channel", channel);
    self.add_element("aps", aps);
    self.add_element("line1", line1);
    self.add_element("line2", line2);
    self.add_element("uptime", uptime);
    self.add_element("face", face);
    self.add_element("friend_name", friend_name);
    self.add_element("name", name);
    self.add_element("status", status);
    self.add_element("shakes", shakes);
    self.add_element("mode", mode);
  }

  fn make_channel_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> LabeledValue {
    LabeledValue::new(
      pos,
      "CH".to_string(),
      "-".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 10.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: None,
        wrap: false,
      },
    )
  }

  fn make_aps_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> LabeledValue {
    LabeledValue::new(
      pos,
      "APS".to_string(),
      "0".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 10.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: None,
        wrap: false,
      },
    )
  }

  fn make_uptime_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> LabeledValue {
    LabeledValue::new(
      pos,
      "UP".to_string(),
      "00:00:00".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 10.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: None,
        wrap: false,
      },
    )
  }

  const fn make_line_widget(pos: ((f32, f32), (f32, f32)), color: Rgba<u8>) -> Line {
    Line::new(pos, image::Rgba(color.0), 2)
  }

  fn make_face_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> TextWidget {
    TextWidget::new(
      pos,
      face_to_string(&FaceType::Awake),
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 40.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: None,
        wrap: false,
      },
    )
  }

  fn make_friend_name_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> TextWidget {
    TextWidget::new(
      pos,
      "",
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 10.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: None,
        wrap: false,
      },
    )
  }

  fn make_name_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> TextWidget {
    TextWidget::new(
      pos,
      format!("{}>", config().main.name),
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 10.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: None,
        wrap: false,
      },
    )
  }

  fn make_status_widget(
    pos: (u32, u32),
    fontname: &str,
    color: Rgba<u8>,
    voice: &Voice,
  ) -> TextWidget {
    TextWidget::new(
      pos,
      voice.default_line(),
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 10.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: Some(40),
        wrap: true,
      },
    )
  }

  fn make_shakes_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> LabeledValue {
    LabeledValue::new(
      pos,
      "PWND".to_string(),
      "0 (00)".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 10.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: None,
        wrap: false,
      },
    )
  }

  fn make_mode_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> TextWidget {
    TextWidget::new(
      pos,
      "AUTO",
      TextStyle {
        font: fontname.to_string(),
        color: image::Rgba(color.0),
        size: 10.0,
        weight: cosmic_text::Weight::EXTRA_BOLD,
        max_length: None,
        wrap: false,
      },
    )
  }

  pub fn has_element(&self, key: &str) -> bool {
    match self.state.lock().unwrap_or_else(PoisonError::into_inner).elements.lock() {
      Ok(state) => state.contains_key(key),
      Err(e) => {
        eprintln!("Failed to lock state elements: {e}");
        false
      }
    }
  }

  pub fn add_element<T: Widget + 'static>(&self, key: &str, elem: T) {
    let wrapped_elem: Arc<Mutex<dyn Widget>> = Arc::new(Mutex::new(elem));

    self
      .state
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .add_element(key, wrapped_elem);
  }

  pub fn on_state_change<F>(&self, key: &str, callback: F)
  where
    F: Fn(StateValue, StateValue) + Send + Sync + 'static,
  {
    self
      .state
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .add_listener(key, callback);
  }

  pub fn on_render(&mut self, callback: Option<fn(&RgbaImage)>) {
    if let Some(cb) = callback
      && !self.render_callbacks.contains(&cb)
    {
      match Arc::get_mut(&mut self.render_callbacks) {
        Some(callbacks) => callbacks.push(cb),
        None => eprintln!("Failed to add render callback, multiple references exist"),
      }
    }
  }

  pub fn on_starting(&self) {
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_starting()),
    );
    self.set("face", StateValue::Face(FaceType::Awake));
  }

  pub fn on_manual_mode(&self, last_session: &LastSession) {
    self.set("mode", StateValue::Text("MANU".into()));
    self.set(
      "face",
      StateValue::Face(if last_session.epochs > 3 && last_session.handshakes == 0 {
        FaceType::Sad
      } else {
        FaceType::Happy
      }),
    );

    self.set(
      "status",
      StateValue::Text(
        self
          .voice
          .lock()
          .unwrap_or_else(PoisonError::into_inner)
          .on_last_session_data(last_session),
      ),
    );
    self.set("epoch", StateValue::Text(last_session.epochs.to_string()));
    self.set("uptime", StateValue::Text(last_session.duration.clone()));
    self.set("channel", StateValue::Text("-".into()));
    self.set("aps", StateValue::Text(last_session.associated.to_string()));
    self.set(
      "shakes",
      StateValue::Text(format!(
        "{} ({:02})",
        last_session.handshakes,
        total_unique_handshakes(&config().main.handshakes_path)
      )),
    );

    //self.set_closest_peer(last_session.last_peer.as_ref(), last_session.peers);
    self.update(None, None);
  }

  pub fn on_keys_generation(&self) {
    self.set("face", StateValue::Face(FaceType::Awake));
    self.set(
      "status",
      StateValue::Text(
        self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_keys_generation(),
      ),
    );
    self.update(None, None);
  }

  pub fn on_normal(&self) {
    self.set("face", StateValue::Face(FaceType::Awake));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_normal()),
    );
    self.update(None, None);
  }

  pub const fn on_new_peer(&self, _peer: &Peer) {
    // TODO
  }

  pub fn on_lost_peer(&self, _peer: &Peer) {
    self.set("face", StateValue::Face(FaceType::Lonely));
    // self.set("status", StateValue::Text(self.voice.on_lost_peer(peer)));
  }

  pub fn on_free_channel(&self, channel: u8) {
    self.set("face", StateValue::Face(FaceType::Smart));
    self.set(
      "status",
      StateValue::Text(
        self
          .voice
          .lock()
          .unwrap_or_else(PoisonError::into_inner)
          .on_free_channel(channel),
      ),
    );
    self.update(None, None);
  }

  pub fn on_reading_logs(&self, lines: u64) {
    self.set("face", StateValue::Face(FaceType::Smart));
    self.set(
      "status",
      StateValue::Text(
        self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_reading_logs(lines),
      ),
    );
    self.update(None, None);
  }

  pub fn on_shutdown(&mut self) {
    self.set("face", StateValue::Face(FaceType::Sleep));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_shutdown()),
    );
    self.update(None, None);
    self.frozen = true;
  }

  pub fn on_bored(&self) {
    self.set("face", StateValue::Face(FaceType::Bored));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_bored()),
    );
    self.update(None, None);
  }

  pub fn on_sad(&self) {
    self.set("face", StateValue::Face(FaceType::Sad));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_sad()),
    );
    self.update(None, None);
  }

  pub fn on_angry(&self) {
    self.set("face", StateValue::Face(FaceType::Angry));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_angry()),
    );
    self.update(None, None);
  }

  pub fn on_motivated(&self) {
    self.set("face", StateValue::Face(FaceType::Motivated));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_motivated()),
    );
    self.update(None, None);
  }

  pub fn on_demotivated(&self) {
    self.set("face", StateValue::Face(FaceType::Demotivated));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_demotivated()),
    );
    self.update(None, None);
  }

  pub fn on_excited(&self) {
    self.set("face", StateValue::Face(FaceType::Excited));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_excited()),
    );
    self.update(None, None);
  }

  pub fn on_assoc(&self, ap: &AccessPoint) {
    self.set("face", StateValue::Face(FaceType::Intense));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_assoc(ap)),
    );
    self.update(None, None);
  }

  pub fn on_deauth(&self, who: &Station) {
    self.set("face", StateValue::Face(FaceType::Cool));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_deauth(who)),
    );
    self.update(None, None);
  }

  pub fn on_miss(&self, who: &AccessPoint) {
    self.set("face", StateValue::Face(FaceType::Sad));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_miss(&who.mac)),
    );
    self.update(None, None);
  }

  pub fn on_grateful(&self) {
    self.set("face", StateValue::Face(FaceType::Grateful));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_grateful()),
    );
    self.update(None, None);
  }

  pub fn on_lonely(&self) {
    self.set("face", StateValue::Face(FaceType::Lonely));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_lonely()),
    );
    self.update(None, None);
  }

  pub fn on_handshakes(&self, count: u32) {
    self.set("face", StateValue::Face(FaceType::Happy));
    self.set(
      "status",
      StateValue::Text(
        self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_handshakes(count),
      ),
    );
    self.update(None, None);
  }

  pub fn on_unread_messages(&self, count: u32) {
    self.set("face", StateValue::Face(FaceType::Excited));
    self.set(
      "status",
      StateValue::Text(
        self
          .voice
          .lock()
          .unwrap_or_else(PoisonError::into_inner)
          .on_unread_messages(count),
      ),
    );
    self.update(None, None);
    std::thread::sleep(Duration::from_millis(100));
  }

  pub fn on_uploading(&self, to: &str) {
    let faces = [
      FaceType::Upload,
      FaceType::Upload1,
      FaceType::Upload2,
    ];

    self.set("face", StateValue::Face(faces[fastrand::usize(..faces.len())]));

    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_uploading(to)),
    );

    self.update(Some(true), None);
  }

  pub fn on_rebooting(&self) {
    self.set("face", StateValue::Face(FaceType::Broken));
    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_rebooting()),
    );
    self.update(None, None);
  }

  pub fn on_custom(&self, text: &str) {
    self.set("face", StateValue::Face(FaceType::Debug));

    self.set(
      "status",
      StateValue::Text(self.voice.lock().unwrap_or_else(PoisonError::into_inner).custom(text)),
    );

    self.update(None, None);
  }

  pub fn get(&self, key: &str) -> Option<Arc<Mutex<dyn Widget>>> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner).get(key)
  }

  pub fn set(&self, key: &str, value: StateValue) {
    let value = match value {
      StateValue::Face(face) => StateValue::Text(face_to_string(&face)),
      _ => value,
    };

    self.state.lock().unwrap_or_else(PoisonError::into_inner).set(key, value);
  }

  pub fn is_normal(&self) -> bool {
    let special_moods = [
      FaceType::Intense,
      FaceType::Cool,
      FaceType::Bored,
      FaceType::Happy,
      FaceType::Excited,
      FaceType::Motivated,
      FaceType::Demotivated,
      FaceType::Smart,
      FaceType::Lonely,
      FaceType::Sad,
    ];

    let special_text: std::collections::HashSet<String> =
      special_moods.iter().map(face_to_string).collect();

    self.get("face").is_none_or(|face_arc| {
      let face = face_arc.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

      match face.get_value() {
        StateValue::Face(f) => !special_moods.contains(&f),
        StateValue::Text(ref s) => !special_text.contains(s),
        _ => true,
      }
    })
  }

  pub async fn wait(&self, mut secs: f64, sleeping: bool, automata: &Automata) {
    let was_normal = self.is_normal();

    let part = secs / 10.0;

    for step in 0..10 {
      if was_normal || step > 5 {
        if sleeping {
          if secs > 1.0 {
            self.set("face", StateValue::Face(FaceType::Sleep));

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            self.set(
              "status",
              StateValue::Text(
                self
                  .voice
                  .lock()
                  .unwrap_or_else(PoisonError::into_inner)
                  .on_napping(secs as u64),
              ),
            );
          } else {
            self.set("face", StateValue::Face(FaceType::Sleep2));

            self.set(
              "status",
              StateValue::Text(
                self.voice.lock().unwrap_or_else(PoisonError::into_inner).on_awakening(),
              ),
            );
          }
        } else {
          #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
          self.set(
            "status",
            StateValue::Text(
              self
                .voice
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .on_waiting(secs as u64),
            ),
          );

          let good_mood = automata.in_good_mood();

          let face = if step % 2 == 0 {
            if good_mood { FaceType::LookRHappy } else { FaceType::LookR }
          } else if good_mood {
            FaceType::LookLHappy
          } else {
            FaceType::LookL
          };

          self.set("face", StateValue::Face(face));
        }
      }

      tokio::time::sleep(Duration::from_secs_f64(part)).await;

      secs -= part;
    }

    self.on_normal();
  }

  pub fn update(&self, force: Option<bool>, new_data: Option<HashMap<String, StateValue>>) {
    let force = force.unwrap_or(false);

    if let Some(new_data) = new_data
      && let Ok(state_guard) = self.state.lock()
    {
      for (key, value) in new_data {
        let _ = catch_unwind(|| state_guard.set(&key, value));
      }
    }

    if self.frozen {
      return;
    }

    let changes = self
      .state
      .lock()
      .map_or_else(|_| Vec::new(), |state_guard| state_guard.changes(&self.ignore_changes));

    if force || !changes.is_empty() {
      let mut canvas = RgbaImage::from_pixel(self.width, self.height, self.background_color);

      if let Ok(state_guard) = self.state.lock() {
        for widget in state_guard.items().values() {
          if let Ok(widget_guard) = widget.lock() {
            widget_guard.draw(&mut canvas);
          }
        }

        let _ = catch_unwind(|| frame::update_frame(&canvas));

        let cbs = Arc::clone(&self.render_callbacks);
        for cb in &*cbs {
          let _ = catch_unwind(|| cb(&canvas));
        }

        let _ = catch_unwind(|| state_guard.reset());
      }
    }
  }
}

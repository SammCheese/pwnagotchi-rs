use std::{
  collections::HashMap,
  fmt::Write,
  panic::catch_unwind,
  sync::{Arc, Mutex, PoisonError},
  time::Duration,
};

use pwnagotchi_hw::display::base::DisplayTrait;
use pwnagotchi_shared::{
  config::config,
  logger::LOGGER,
  mesh::peer::Peer,
  models::net::{AccessPoint, Station},
  sessions::lastsession::LastSession,
  traits::{
    automata::AgentObserver,
    ui::{ViewTrait, Widget},
    voice::VoiceTrait,
  },
  types::ui::{FaceType, StateValue},
  utils::general::{format_duration_human, total_unique_handshakes},
};
use rgb::Rgba;
use tiny_skia::PixmapMut as RgbaImage;

use crate::{
  ui::{
    components::{LabeledValue, Line, TextStyle, TextWidget},
    fonts::STATUS_FONT_NAME,
    state::State,
  },
  web::frame,
};

const WHITE: Rgba<u8> = Rgba { r: 255, g: 255, b: 255, a: 255 };
const BLACK: Rgba<u8> = Rgba { r: 0, g: 0, b: 0, a: 255 };

pub struct View {
  pub voice: Arc<dyn VoiceTrait + Send + Sync>,
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

#[async_trait::async_trait]
impl ViewTrait for View {
  fn on_state_change(
    &self,
    key: &str,
    callback: Box<dyn Fn(StateValue, StateValue) + Send + Sync>,
  ) {
    self
      .state
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .add_listener(key, callback);
  }

  async fn wait(
    &self,
    mut secs: f64,
    sleeping: bool,
    automata: &Arc<dyn AgentObserver + Send + Sync>,
  ) {
    let was_normal = self.is_normal();

    let part = secs / 10.0;

    for step in 0..10 {
      if was_normal || step > 5 {
        if sleeping {
          if secs > 1.0 {
            self.set("face", StateValue::Text(FaceType::Sleep.to_string()));

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            self.set("status", StateValue::Text(self.voice.on_napping(secs as u64)));
          } else {
            self.set("face", StateValue::Text(FaceType::Sleep2.to_string()));

            self.set("status", StateValue::Text(self.voice.on_awakening()));
          }
        } else {
          #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
          self.set("status", StateValue::Text(self.voice.on_waiting(secs as u64)));

          let good_mood = automata.in_good_mood();

          let face = if step % 2 == 0 {
            if good_mood { FaceType::LookRHappy } else { FaceType::LookR }
          } else if good_mood {
            FaceType::LookLHappy
          } else {
            FaceType::LookL
          };

          self.set("face", StateValue::Text(face.to_string()));
        }
      }

      tokio::time::sleep(Duration::from_secs_f64(part)).await;

      secs -= part;
    }

    self.on_normal();
  }

  fn on_starting(&self) {
    self.set("status", StateValue::Text(self.voice.on_starting()));
    self.set("face", StateValue::Text(FaceType::Awake.to_string()));
  }

  fn on_manual_mode(&self, last_session: &LastSession) {
    let Some(session) = last_session.stats.as_ref() else {
      eprintln!("Warning: last_session.stats is None");
      return;
    };

    self.set("mode", StateValue::Text("MANU".into()));
    self.set(
      "face",
      StateValue::Text(if session.epochs.epochs > 3 && session.handshakes == 0 {
        FaceType::Sad.to_string()
      } else {
        FaceType::Happy.to_string()
      }),
    );

    self.set("status", StateValue::Text(self.voice.on_last_session_data(last_session)));
    self.set("epoch", StateValue::Text(session.epochs.epochs.to_string()));
    let duration = Duration::from_secs(session.duration_secs.unwrap_or(0));
    self.set("uptime", StateValue::Text(format_duration_human(duration)));
    self.set("channel", StateValue::Text("-".into()));
    self.set("aps", StateValue::Text(session.associated.to_string()));
    self.set(
      "shakes",
      StateValue::Text(format!(
        "{} ({:02})",
        session.handshakes,
        total_unique_handshakes(&config().bettercap.handshakes)
      )),
    );

    #[allow(clippy::cast_possible_truncation)]
    self.set_closest_peer(session.peers.last_peer.as_ref(), session.peers.peers as u32);
    self.update(None, None);
  }

  fn set_closest_peer(&self, peer: Option<&Peer>, total_peers: u32) {
    if let Some(peer) = peer {
      let num_bars = if peer.rssi >= -67 {
        4
      } else if peer.rssi >= -70 {
        3
      } else if peer.rssi >= -80 {
        2
      } else {
        1
      };

      let mut name = "▌".repeat(num_bars);
      name += &"│".repeat(4 - num_bars);
      let _ = write!(name, " {} {} ({})", peer.name(), peer.pwnd_run(), peer.pwnd_total());

      if total_peers > 1 {
        if total_peers > 9000 {
          name += " of over 9000";
        } else {
          let _ = write!(name, " of {total_peers}");
        }
      }

      self.set("friend_face", StateValue::Text(peer.face()));
      self.set("friend_name", StateValue::Text(name));
    } else {
      self.set("friend_name", StateValue::Text(String::new()));
      self.set("friend_face", StateValue::Text(String::new()));
    }
    self.update(None, None);
  }

  fn on_new_peer(&self, peer: &Peer) {
    let face = if peer.is_first_encounter() {
      *fastrand::choice(&[FaceType::Awake, FaceType::Cool]).unwrap_or(&FaceType::Awake)
    } else if peer.is_good_friend() {
      *fastrand::choice(&[
        FaceType::Motivated,
        FaceType::Friend,
        FaceType::Happy,
      ])
      .unwrap_or(&FaceType::Friend)
    } else {
      let faces = [
        FaceType::Excited,
        FaceType::Smart,
        FaceType::Happy,
      ];
      *fastrand::choice(&faces).unwrap_or(&FaceType::Excited)
    };
    self.set("face", StateValue::Text(face.to_string()));
    self.set("status", StateValue::Text(self.voice.on_new_peer(peer)));
    self.update(None, None);
    std::thread::sleep(std::time::Duration::from_secs(3));
  }

  fn on_keys_generation(&self) {
    self.set("face", StateValue::Text(FaceType::Awake.to_string()));
    self.set("status", StateValue::Text(self.voice.on_keys_generation()));
    self.update(None, None);
  }

  fn on_normal(&self) {
    self.set("face", StateValue::Text(FaceType::Awake.to_string()));
    self.set("status", StateValue::Text(self.voice.on_normal()));
    self.update(None, None);
  }

  fn on_lost_peer(&self, peer: &Peer) {
    self.set("face", StateValue::Text(FaceType::Lonely.to_string()));
    self.set("status", StateValue::Text(self.voice.on_lost_peer(peer)));
    self.update(None, None);
  }

  fn on_free_channel(&self, channel: u8) {
    self.set("face", StateValue::Text(FaceType::Smart.to_string()));
    self.set("status", StateValue::Text(self.voice.on_free_channel(channel)));
    self.update(None, None);
  }

  fn on_reading_logs(&self, lines: u64) {
    self.set("face", StateValue::Text(FaceType::Smart.to_string()));
    self.set("status", StateValue::Text(self.voice.on_reading_logs(lines)));
    self.update(None, None);
  }

  fn on_shutdown(&mut self) {
    self.set("face", StateValue::Text(FaceType::Sleep.to_string()));
    self.set("status", StateValue::Text(self.voice.on_shutdown()));
    self.update(None, None);
    self.frozen = true;
  }

  fn on_bored(&self) {
    self.set("face", StateValue::Text(FaceType::Bored.to_string()));
    self.set("status", StateValue::Text(self.voice.on_bored()));
    self.update(None, None);
  }

  fn on_sad(&self) {
    self.set("face", StateValue::Text(FaceType::Sad.to_string()));
    self.set("status", StateValue::Text(self.voice.on_sad()));
    self.update(None, None);
  }

  fn on_angry(&self) {
    self.set("face", StateValue::Text(FaceType::Angry.to_string()));
    self.set("status", StateValue::Text(self.voice.on_angry()));
    self.update(None, None);
  }

  fn on_motivated(&self) {
    self.set("face", StateValue::Text(FaceType::Motivated.to_string()));
    self.set("status", StateValue::Text(self.voice.on_motivated()));
    self.update(None, None);
  }

  fn on_demotivated(&self) {
    self.set("face", StateValue::Text(FaceType::Demotivated.to_string()));
    self.set("status", StateValue::Text(self.voice.on_demotivated()));
    self.update(None, None);
  }

  fn on_excited(&self) {
    self.set("face", StateValue::Text(FaceType::Excited.to_string()));
    self.set("status", StateValue::Text(self.voice.on_excited()));
    self.update(None, None);
  }

  fn on_assoc(&self, ap: &AccessPoint) {
    self.set("face", StateValue::Text(FaceType::Intense.to_string()));
    self.set("status", StateValue::Text(self.voice.on_assoc(ap)));
    self.update(None, None);
  }

  fn on_deauth(&self, who: &Station) {
    self.set("face", StateValue::Text(FaceType::Cool.to_string()));
    self.set("status", StateValue::Text(self.voice.on_deauth(who)));
    self.update(None, None);
  }

  fn on_miss(&self, who: &AccessPoint) {
    self.set("face", StateValue::Text(FaceType::Sad.to_string()));
    self.set("status", StateValue::Text(self.voice.on_miss(&who.mac)));
    self.update(None, None);
  }

  fn on_grateful(&self) {
    self.set("face", StateValue::Text(FaceType::Grateful.to_string()));
    self.set("status", StateValue::Text(self.voice.on_grateful()));
    self.update(None, None);
  }

  fn on_lonely(&self) {
    self.set("face", StateValue::Text(FaceType::Lonely.to_string()));
    self.set("status", StateValue::Text(self.voice.on_lonely()));
    self.update(None, None);
  }

  fn on_handshakes(&self, count: u32) {
    self.set("face", StateValue::Text(FaceType::Happy.to_string()));
    self.set("status", StateValue::Text(self.voice.on_handshakes(count)));
    self.update(None, None);
  }

  fn on_unread_messages(&self, count: u32) {
    self.set("face", StateValue::Text(FaceType::Excited.to_string()));
    self.set("status", StateValue::Text(self.voice.on_unread_messages(count)));
    self.update(None, None);
    std::thread::sleep(Duration::from_millis(100));
  }

  fn on_uploading(&self, to: &str) {
    let faces = [
      FaceType::Upload,
      FaceType::Upload1,
      FaceType::Upload2,
    ];

    self.set("face", StateValue::Text(faces[fastrand::usize(..faces.len())].to_string()));
    self.set("status", StateValue::Text(self.voice.on_uploading(to)));
    self.update(Some(true), None);
  }

  fn on_rebooting(&self) {
    self.set("face", StateValue::Text(FaceType::Broken.to_string()));
    self.set("status", StateValue::Text(self.voice.on_rebooting()));
    self.update(None, None);
  }

  fn on_custom(&self, text: &str) {
    self.set("face", StateValue::Text(FaceType::Debug.to_string()));
    self.set("status", StateValue::Text(self.voice.custom(text)));
    self.update(None, None);
  }

  fn get(&self, key: &str) -> Option<Arc<Mutex<dyn Widget>>> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner).get(key)
  }

  fn set(&self, key: &str, value: StateValue) {
    self.state.lock().unwrap_or_else(PoisonError::into_inner).set(key, value);
  }

  fn is_normal(&self) -> bool {
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
      special_moods.iter().map(FaceType::to_string).collect();

    self.get("face").is_none_or(|face_arc| {
      let face = face_arc.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

      match face.get_value() {
        StateValue::Text(ref s) => !special_text.contains(s),
        _ => true,
      }
    })
  }

  fn update(&self, force: Option<bool>, new_data: Option<HashMap<String, StateValue>>) {
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
      let pixel_count = (self.width * self.height) as usize;
      let mut buffer = vec![
        self.background_color.r,
        self.background_color.g,
        self.background_color.b,
        self.background_color.a,
      ]
      .into_iter()
      .cycle()
      .take(pixel_count * 4)
      .collect::<Vec<u8>>();
      let mut canvas = RgbaImage::from_bytes(&mut buffer, self.width, self.height)
        .expect("Failed to create canvas from buffer");

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

  async fn start_render_loop(&self) {
    let delay = 1.0 / f64::from(config().ui.fps);
    loop {
      self.update(None, None);
      tokio::time::sleep(Duration::from_secs_f64(delay)).await;
    }
  }
}

impl View {
  pub fn new(
    display: &Arc<dyn DisplayTrait + Send + Sync>,
    voice: &Arc<dyn VoiceTrait + Send + Sync>,
  ) -> Self {
    let inverted = config().ui.inverted;
    let background_color = if inverted { BLACK } else { WHITE };
    let foreground_color = if inverted { WHITE } else { BLACK };

    let layout = Arc::clone(display);

    let view = Self {
      display: Arc::clone(display),
      voice: Arc::clone(voice),
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
    let status =
      { Self::make_status_widget(layout.status.pos, fontname, self.foreground_color, &self.voice) };
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
      TextStyle {
        font: fontname.to_string(),
        color,
        size: 10.0,
        weight: cosmic_text::Weight::EXTRA_BOLD,
        max_length: None,
        wrap: false,
      },
      "-".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color,
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
      TextStyle {
        font: fontname.to_string(),
        color,
        size: 10.0,
        weight: cosmic_text::Weight::EXTRA_BOLD,
        max_length: None,
        wrap: false,
      },
      "0".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color,
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
      TextStyle {
        font: fontname.to_string(),
        color,
        size: 10.0,
        weight: cosmic_text::Weight::EXTRA_BOLD,
        max_length: None,
        wrap: false,
      },
      "00:00:00".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color,
        size: 10.0,
        weight: cosmic_text::Weight::NORMAL,
        max_length: None,
        wrap: false,
      },
    )
  }

  const fn make_line_widget(pos: ((f32, f32), (f32, f32)), color: Rgba<u8>) -> Line {
    Line::new(pos, color, 2)
  }

  fn make_face_widget(pos: (u32, u32), fontname: &str, color: Rgba<u8>) -> TextWidget {
    TextWidget::new(
      pos,
      FaceType::Awake.to_string(),
      TextStyle {
        font: fontname.to_string(),
        color,
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
      String::new(),
      TextStyle {
        font: fontname.to_string(),
        color,
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
        color,
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
    voice: &Arc<dyn VoiceTrait + Send + Sync>,
  ) -> TextWidget {
    TextWidget::new(
      pos,
      voice.default_line(),
      TextStyle {
        font: fontname.to_string(),
        color,
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
      TextStyle {
        font: fontname.to_string(),
        color,
        size: 10.0,
        weight: cosmic_text::Weight::EXTRA_BOLD,
        max_length: None,
        wrap: false,
      },
      "0 (00)".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color,
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
      "AUTO".to_string(),
      TextStyle {
        font: fontname.to_string(),
        color,
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
}

use std::{ collections::HashMap, thread::sleep, time::Duration };

use image::ImageBuffer;
use imageproc::{definitions::Image, drawing::Canvas};
use rand::Rng;

use crate::core::{
  agent::{ AccessPoint, Peer, Station },
  config::config,
  session::LastSession,
  ui::state::{ State, StateValue },
  utils::{ total_unique_handshakes },
  voice::Voice,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

const WHITE: i32 = 0x00;
const BLACK: i32 = 0xff;

pub struct View {
  pub voice: Voice,
  pub state: State,
  pub inverted: bool,
  pub background_color: i32,
  pub foreground_color: i32,
  pub frozen: bool,
  pub ignore_changes: Vec<&'static str>,
  pub render_callbacks: Vec<fn()>,
}

impl Default for View {
  fn default() -> Self {
    let inverted = false;
    let background_color = WHITE;
    let foreground_color = BLACK;
    Self {
      voice: Voice::new(),
      state: State::new(),
      inverted,
      background_color,
      foreground_color,
      frozen: false,
      ignore_changes: Vec::new(),
      render_callbacks: Vec::new(),
    }
  }
}

impl View {
  pub fn new() -> Self {
    let inverted = config().ui.inverted;
    let background_color = if inverted { BLACK } else { WHITE };
    let foreground_color = if inverted { WHITE } else { BLACK };
    let state = State::new();
    Self {
      voice: Voice::new(),
      state,
      inverted,
      background_color,
      foreground_color,
      frozen: false,
      ignore_changes: Vec::new(),
      render_callbacks: Vec::new(),
    }
  }

  pub fn on_state_change<F>(&self, key: &str, callback: F)
    where F: Fn(StateValue, StateValue) + Send + Sync + 'static
  {
    self.state.add_listener(key, callback);
  }

  pub fn on_render(&mut self, callback: Option<fn()>) {
    if let Some(cb) = callback
      && !self.render_callbacks.contains(&cb) {
        self.render_callbacks.push(cb);
      }
  }

  fn refresh_handler(&self) {
    let delay = 1.0 / f64::from(config().ui.fps);
    loop {
      self.update(None, None);
      sleep(Duration::from_secs_f64(delay));
    }
  }

  pub fn on_starting(&self) {
    self.set(
      "status",
      StateValue::Text(self.voice.on_starting() + &format!("\n(v{}", env!("CARGO_PKG_VERSION")))
    );
    self.set("face", StateValue::Face(FaceType::Awake));
  }

  pub fn on_manual_mode(&self, last_session: &LastSession) {
    self.set("mode", StateValue::Text("MANU".into()));
    self.set(
      "face",
      StateValue::Face(
        if last_session.epochs > 3 && last_session.handshakes == 0 {
          FaceType::Sad
        } else {
          FaceType::Happy
        }
      )
    );
    self.set("status", StateValue::Text(self.voice.on_last_session_data(last_session)));
    self.set("epoch", StateValue::Text(last_session.epochs.to_string()));
    self.set("uptime", StateValue::Text(last_session.duration.to_string()));
    self.set("channel", StateValue::Text("-".into()));
    self.set("aps", StateValue::Text(last_session.associated.to_string()));
    self.set(
      "shakes",
      StateValue::Text(
        format!(
          "{} ({})",
          total_unique_handshakes(&config().main.handshakes_path),
          last_session.handshakes
        )
      )
    );
    //self.set_closest_peer(last_session.last_peer.as_ref(), last_session.peers);
    self.update(None, None);
  }

  pub fn on_keys_generation(&self) {
    self.set("face", StateValue::Face(FaceType::Awake));
    self.set("status", StateValue::Text(self.voice.on_keys_generation()));
    self.update(None, None);
  }

  pub fn on_normal(&self) {
    self.set("face", StateValue::Face(FaceType::Awake));
    self.set("status", StateValue::Text(self.voice.on_normal()));
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
    self.set("status", StateValue::Text(self.voice.on_free_channel(channel)));
    self.update(None, None);
  }

  pub fn on_reading_logs(&self, lines: u64) {
    self.set("face", StateValue::Face(FaceType::Smart));
    self.set("status", StateValue::Text(self.voice.on_reading_logs(lines)));
    self.update(None, None);
  }

  pub fn on_shutdown(&mut self) {
    self.set("face", StateValue::Face(FaceType::Sleep));
    self.set("status", StateValue::Text(self.voice.on_shutdown()));
    self.update(None, None);
    self.frozen = true;
  }

  pub fn on_bored(&self) {
    self.set("face", StateValue::Face(FaceType::Bored));
    self.set("status", StateValue::Text(self.voice.on_bored()));
    self.update(None, None);
  }

  pub fn on_sad(&self) {
    self.set("face", StateValue::Face(FaceType::Sad));
    self.set("status", StateValue::Text(self.voice.on_sad()));
    self.update(None, None);
  }

  pub fn on_angry(&self) {
    self.set("face", StateValue::Face(FaceType::Angry));
    self.set("status", StateValue::Text(self.voice.on_angry()));
    self.update(None, None);
  }

  pub fn on_motivated(&self) {
    self.set("face", StateValue::Face(FaceType::Motivated));
    self.set("status", StateValue::Text(self.voice.on_motivated()));
    self.update(None, None);
  }

  pub fn on_demotivated(&self) {
    self.set("face", StateValue::Face(FaceType::Demotivated));
    self.set("status", StateValue::Text(self.voice.on_demotivated()));
    self.update(None, None);
  }

  pub fn on_excited(&self) {
    self.set("face", StateValue::Face(FaceType::Excited));
    self.set("status", StateValue::Text(self.voice.on_excited()));
    self.update(None, None);
  }

  pub fn on_assoc(&self, ap: &AccessPoint) {
    self.set("face", StateValue::Face(FaceType::Intense));
    self.set("status", StateValue::Text(self.voice.on_assoc(ap.clone())));
    self.update(None, None);
  }

  pub fn on_deauth(&self, who: &Station) {
    self.set("face", StateValue::Face(FaceType::Angry));
    self.set("status", StateValue::Text(self.voice.on_deauth(who)));
    self.update(None, None);
  }

  pub fn on_miss(&self, who: &Station) {
    self.set("face", StateValue::Face(FaceType::Sad));
    self.set("status", StateValue::Text(self.voice.on_miss(&who.mac)));
    self.update(None, None);
  }

  pub fn on_grateful(&self) {
    self.set("face", StateValue::Face(FaceType::Grateful));
    self.set("status", StateValue::Text(self.voice.on_grateful()));
    self.update(None, None);
  }

  pub fn on_lonely(&self) {
    self.set("face", StateValue::Face(FaceType::Lonely));
    self.set("status", StateValue::Text(self.voice.on_lonely()));
    self.update(None, None);
  }

  pub fn on_handshakes(&self, count: u32) {
    self.set("face", StateValue::Face(FaceType::Happy));
    self.set("status", StateValue::Text(self.voice.on_handshakes(count)));
    self.update(None, None);
  }

  pub fn on_unread_messages(&self, count: u32) {
    self.set("face", StateValue::Face(FaceType::Excited));
    self.set("status", StateValue::Text(self.voice.on_unread_messages(count)));
    self.update(None, None);
    sleep(Duration::from_millis(100));
  }

  pub fn on_uploading(&self, to: &str) {
    let faces = [FaceType::Upload, FaceType::Upload1, FaceType::Upload2];
    self.set("face", StateValue::Face(faces[rand::rng().random_range(0..faces.len())]));
    self.set("status", StateValue::Text(self.voice.on_uploading(to)));
    self.update(Some(true), None);
  }

  pub fn on_rebooting(&self) {
    self.set("face", StateValue::Face(FaceType::Broken));
    self.set("status", StateValue::Text(self.voice.on_rebooting()));
    self.update(None, None);
  }

  pub fn on_custom(&self, text: &str) {
    self.set("face", StateValue::Face(FaceType::Debug));
    self.set("status", StateValue::Text(self.voice.custom(text)));
    self.update(None, None);
  }

  pub fn face_to_string(face: FaceType) -> String {
    let faces = &config().faces;
    let face_str = match face {
      FaceType::LookR => &faces.look_r,
      FaceType::LookL => &faces.look_l,
      FaceType::LookRHappy => &faces.look_r_happy,
      FaceType::LookLHappy => &faces.look_l_happy,
      FaceType::Sleep => &faces.sleep,
      FaceType::Sleep2 => &faces.sleep2,
      FaceType::Awake => &faces.awake,
      FaceType::Bored => &faces.bored,
      FaceType::Intense => &faces.intense,
      FaceType::Cool => &faces.cool,
      FaceType::Happy => &faces.happy,
      FaceType::Grateful => &faces.grateful,
      FaceType::Excited => &faces.excited,
      FaceType::Motivated => &faces.motivated,
      FaceType::Demotivated => &faces.demotivated,
      FaceType::Smart => &faces.smart,
      FaceType::Lonely => &faces.lonely,
      FaceType::Sad => &faces.sad,
      FaceType::Angry => &faces.angry,
      FaceType::Friend => &faces.friend,
      FaceType::Broken => &faces.broken,
      FaceType::Debug => &faces.debug,
      FaceType::Upload => &faces.upload,
      FaceType::Upload1 => &faces.upload1,
      FaceType::Upload2 => &faces.upload2,
    };
    face_str.to_string()
  }

  pub fn get(&self, key: &str) -> Option<StateValue> {
    self.state.get(key)
  }

  pub fn set(&self, key: &str, value: StateValue) {
    self.state.set(key, value);
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

    // iterate and check for special faces
    self.state
      .get("face")
      .filter(|face| {
        let face = match face {
          StateValue::Face(f) => *f,
          _ => {
            return false;
          }
        };
        special_moods.contains(&face)
      })
      .is_none()
  }

  pub fn wait(&self, mut secs: f64, sleeping: Option<bool>) {
    let sleeping = sleeping.unwrap_or(true);
    let was_normal = self.is_normal();
    let part = secs / 10.0;

    for step in 0..10 {
      if was_normal || step > 5 {
        if sleeping {
          if secs > 1.0 {
            self.set("face", StateValue::Face(FaceType::Sleep));
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            self.set("status", StateValue::Text(self.voice.on_napping(secs as u64)));
          } else {
            self.set("face", StateValue::Face(FaceType::Sleep2));
            self.set("status", StateValue::Text(self.voice.on_awakening()));
          }
        } else {
          #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
          self.set("status", StateValue::Text(self.voice.on_waiting(secs as u64)));

          let good_mood = true; //self.agent.emotion.in_good_mood();

          if step % 2 == 0 {
            self.set(
              "face",
              StateValue::Face(if good_mood { FaceType::LookRHappy } else { FaceType::LookR })
            );
          } else {
            self.set(
              "face",
              StateValue::Face(if good_mood { FaceType::LookLHappy } else { FaceType::LookL })
            );
          }
        }
      }
      std::thread::sleep(std::time::Duration::from_secs_f64(part));
      secs -= part;
    }

    self.on_normal();
  }

  pub fn update(&self, force: Option<bool>, new_data: Option<HashMap<String, StateValue>>) {
    let force = force.unwrap_or(false);
    if let Some(new_data) = new_data {
      for (key, value) in new_data {
        self.set(&key, value);
      }
    }

    if self.frozen {
      return;
    }

    let state = self.state.clone();
    let changes = state.changes(&self.ignore_changes);

    if force || !changes.is_empty() {
      //let canvas = ImageBuffer::new(200, 200);
      //let drawer = canvas.draw();

      self.state.reset();
    }

    // TODO
  }
}

use image::{Rgba, RgbaImage};
use rand::Rng;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::core::{
    automata::Automata,
    config::config,
    log::LOGGER,
    models::net::{AccessPoint, Peer, Station},
    session::LastSession,
    ui::{
        components::{LabeledValue, Line, TextWidget, Widget},
        fonts::STATUS_FONT_NAME,
        old::{
            hw::base::{DisplayTrait, get_display_from_config},
            web::frame,
        },
        state::{State, StateValue},
    },
    utils::{face_to_string, total_unique_handshakes},
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

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);

pub struct View {
    pub voice: Voice,
    pub state: State,
    pub display: Box<dyn DisplayTrait>,
    pub inverted: bool,
    pub background_color: Rgba<u8>,
    pub foreground_color: Rgba<u8>,
    pub frozen: bool,
    pub ignore_changes: Vec<&'static str>,
    pub render_callbacks: Vec<fn(&RgbaImage)>,
    pub width: u32,
    pub height: u32,
}

impl View {
    pub fn new(display: Box<dyn DisplayTrait>) -> Self {
        let inverted = config().ui.inverted;
        let background_color = if inverted { BLACK } else { WHITE };
        let foreground_color = if inverted { WHITE } else { BLACK };
        let mut view = Self {
            display,
            voice: Voice::new(),
            state: State::new(),
            inverted,
            background_color,
            foreground_color,
            frozen: false,
            ignore_changes: Vec::new(),
            render_callbacks: Vec::new(),
            width: 0,
            height: 0,
        };
        view.initialize();
        view
    }

    fn initialize(&mut self) {
        self.populate_state();
        self.width = self.display.layout().width;
        self.height = self.display.layout().height;
        if config().ui.fps > 0.0 {
            self.start_render_loop();
        } else {
            self.ignore_changes.push("uptime");
            self.ignore_changes.push("name");
            LOGGER.log_warning("UI", "FPS set to 0, Display only updates for major changes");
        }
    }

    fn start_render_loop(&self) {
        let view = Arc::new(self.clone_for_render_loop());
        tokio::spawn(async move {
            let delay = 1.0 / f64::from(config().ui.fps);
            loop {
                view.update(None, None);
                tokio::time::sleep(Duration::from_secs_f64(delay)).await;
            }
        });
    }

    fn clone_for_render_loop(&self) -> Self {
        Self {
            voice: self.voice.clone(),
            state: self.state.clone(),
            display: get_display_from_config(),
            inverted: self.inverted,
            background_color: self.background_color,
            foreground_color: self.foreground_color,
            frozen: self.frozen,
            ignore_changes: self.ignore_changes.clone(),
            render_callbacks: self.render_callbacks.clone(),
            width: self.width,
            height: self.height,
        }
    }

    fn populate_state(&mut self) {
        let layout = self.display.layout();
        let fontname = STATUS_FONT_NAME;

        let channel = LabeledValue::new(
            layout.channel,
            "CH".to_string(),
            "-".to_string(),
            fontname,
            image::Rgba(self.foreground_color.0),
            10.0,
        );
        let aps = LabeledValue::new(
            layout.aps,
            "APS".to_string(),
            "0".to_string(),
            fontname,
            image::Rgba(self.foreground_color.0),
            10.0,
        );
        let uptime = LabeledValue::new(
            layout.uptime,
            "UP".to_string(),
            "00:00:00".to_string(),
            fontname,
            image::Rgba(self.foreground_color.0),
            10.0,
        );
        let line1 = Line::new(layout.line1, image::Rgba(self.foreground_color.0), 2);
        let line2 = Line::new(layout.line2, image::Rgba(self.foreground_color.0), 2);
        let face = TextWidget::new(
            layout.face,
            face_to_string(FaceType::Awake),
            fontname,
            image::Rgba(self.foreground_color.0),
            40.0,
            cosmic_text::Weight::NORMAL,
            None,
            false,
        );
        let friend_name = TextWidget::new(
            layout.friend_name,
            "",
            fontname,
            image::Rgba(self.foreground_color.0),
            10.0,
            cosmic_text::Weight::NORMAL,
            None,
            false,
        );
        let name = TextWidget::new(
            layout.name,
            format!("{}>", config().main.name),
            fontname,
            image::Rgba(self.foreground_color.0),
            10.0,
            cosmic_text::Weight::NORMAL,
            None,
            false,
        );
        let status = TextWidget::new(
            layout.status.pos,
            self.voice.default_line(),
            fontname,
            image::Rgba(self.foreground_color.0),
            8.0,
            cosmic_text::Weight::NORMAL,
            Some(40),
            true,
        );
        let shakes = LabeledValue::new(
            layout.shakes,
            "PWND".to_string(),
            "0 (00)".to_string(),
            fontname,
            image::Rgba(self.foreground_color.0),
            10.0,
        );
        let mode = TextWidget::new(
            layout.mode,
            "AUTO",
            fontname,
            image::Rgba(self.foreground_color.0),
            10.0,
            cosmic_text::Weight::EXTRA_BOLD,
            None,
            false,
        );

        let mut handle = self
            .state
            .elements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        handle.insert(
            "channel".to_owned(),
            Arc::new(Mutex::new(channel)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "aps".to_owned(),
            Arc::new(Mutex::new(aps)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "line1".to_owned(),
            Arc::new(Mutex::new(line1)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "line2".to_owned(),
            Arc::new(Mutex::new(line2)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "uptime".to_owned(),
            Arc::new(Mutex::new(uptime)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "face".to_owned(),
            Arc::new(Mutex::new(face)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "friend_name".to_owned(),
            Arc::new(Mutex::new(friend_name)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "name".to_owned(),
            Arc::new(Mutex::new(name)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "status".to_owned(),
            Arc::new(Mutex::new(status)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "shakes".to_owned(),
            Arc::new(Mutex::new(shakes)) as Arc<Mutex<dyn Widget>>,
        );
        handle.insert(
            "mode".to_owned(),
            Arc::new(Mutex::new(mode)) as Arc<Mutex<dyn Widget>>,
        );
    }

    pub fn has_element(&self, key: &str) -> bool {
        match self.state.elements.lock() {
            Ok(state) => state.contains_key(key),
            Err(e) => {
                eprintln!("Failed to lock state elements: {e}");
                false
            }
        }
    }

    pub fn add_element<T: Widget + 'static>(&self, key: &str, elem: T) {
        let wrapped_elem: Arc<Mutex<dyn Widget>> = Arc::new(Mutex::new(elem));
        self.state.add_element(key, wrapped_elem);
    }

    pub fn on_state_change<F>(&self, key: &str, callback: F)
    where
        F: Fn(StateValue, StateValue) + Send + Sync + 'static,
    {
        self.state.add_listener(key, callback);
    }

    pub fn on_render(&mut self, callback: Option<fn(&RgbaImage)>) {
        if let Some(cb) = callback
            && !self.render_callbacks.contains(&cb)
        {
            self.render_callbacks.push(cb);
        }
    }

    pub fn on_starting(&self) {
        self.set("status", StateValue::Text(self.voice.on_starting()));
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
            StateValue::Text(self.voice.on_last_session_data(last_session)),
        );
        self.set("epoch", StateValue::Text(last_session.epochs.to_string()));
        self.set("uptime", StateValue::Text(last_session.duration.clone()));
        self.set("channel", StateValue::Text("-".into()));
        self.set("aps", StateValue::Text(last_session.associated.to_string()));
        self.set(
            "shakes",
            StateValue::Text(format!(
                "{} ({:02})",
                total_unique_handshakes(&config().main.handshakes_path),
                last_session.handshakes
            )),
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
        self.set(
            "status",
            StateValue::Text(self.voice.on_free_channel(channel)),
        );
        self.update(None, None);
    }

    pub fn on_reading_logs(&self, lines: u64) {
        self.set("face", StateValue::Face(FaceType::Smart));
        self.set(
            "status",
            StateValue::Text(self.voice.on_reading_logs(lines)),
        );
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
        self.set("status", StateValue::Text(self.voice.on_assoc(&ap.clone())));
        self.update(None, None);
    }

    pub fn on_deauth(&self, who: &Station) {
        self.set("face", StateValue::Face(FaceType::Cool));
        self.set("status", StateValue::Text(self.voice.on_deauth(who)));
        self.update(None, None);
    }

    pub fn on_miss(&self, who: &AccessPoint) {
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
        self.set(
            "status",
            StateValue::Text(self.voice.on_unread_messages(count)),
        );
        self.update(None, None);
        std::thread::sleep(Duration::from_millis(100));
    }

    pub fn on_uploading(&self, to: &str) {
        let faces = [FaceType::Upload, FaceType::Upload1, FaceType::Upload2];
        self.set(
            "face",
            StateValue::Face(faces[rand::rng().random_range(0..faces.len())]),
        );
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

    pub fn get(&self, key: &str) -> Option<Arc<Mutex<dyn Widget>>> {
        self.state.get(key)
    }

    pub fn set(&self, key: &str, value: StateValue) {
        let value = match value {
            StateValue::Face(face) => StateValue::Text(face_to_string(face)),
            _ => value,
        };
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
        // Compare against both Face and the Text string used by the TextWidget
        let special_text: std::collections::HashSet<String> =
            special_moods.iter().map(|f| face_to_string(*f)).collect();

        self.state.get("face").is_none_or(|face_arc| {
            let face = face_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
                            StateValue::Text(self.voice.on_napping(secs as u64)),
                        );
                    } else {
                        self.set("face", StateValue::Face(FaceType::Sleep2));
                        self.set("status", StateValue::Text(self.voice.on_awakening()));
                    }
                } else {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    self.set(
                        "status",
                        StateValue::Text(self.voice.on_waiting(secs as u64)),
                    );
                    let good_mood = automata.in_good_mood();
                    let face = if step % 2 == 0 {
                        if good_mood {
                            FaceType::LookRHappy
                        } else {
                            FaceType::LookR
                        }
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
        if let Some(new_data) = new_data {
            for (key, value) in new_data {
                self.set(&key, value);
            }
        }

        if self.frozen {
            return;
        }

        let changes = self.state.changes(&self.ignore_changes);

        if force || !changes.is_empty() {
            let mut canvas = RgbaImage::new(self.width, self.height);

            for widget in self.state.items().values() {
                let widget = widget
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                widget.draw(&mut canvas);
            }

            let _ = frame::update_frame(&canvas);

            for cb in &self.render_callbacks {
                cb(&canvas);
            }

            self.state.reset();
        }
    }
}

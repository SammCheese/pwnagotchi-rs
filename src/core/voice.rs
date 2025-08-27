use crate::core::{agent::{AccessPoint, Station}, session::LastSession, utils::random_choice};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Voice {}

impl Default for Voice {
  fn default() -> Self {
    Self::new()
  }
    
}


impl Voice {
  pub const fn new() -> Self {
    Self {}
  }

  pub fn custom(&self, msg: &str) -> String {
    msg.to_string()
  }

  pub fn default_line() -> String {
    "ZzzzZZzzzzZzzz".to_string()
  }

  pub fn on_starting(&self) -> String {
    random_choice(&[
      "I'm starting up!",
      "New day, new hunt, new pwns!",
      "Hack the Planet!",
      "No more mister Wi-Fi!!",
      "Pretty fly 4 a Wi-Fi!",
      "Good Pwning!", // Battlestar Galactica
      "Ensign, Engage!", // Star trek
      "Free your Wi-Fi!", // Matrix
      "Chevron Seven, locked.", // Stargate
      "May the Wi-fi be with you", // Star Wars
    ])
  }

  pub fn on_keys_generation(&self) -> String {
    random_choice(&[
      "Generating keys, do not turn off...",
      "Who am I...?",
      "These Keys will come in handy...",
    ])
  }

  pub fn on_normal(&self) -> String {
    random_choice(&[
      "",
      "...",
    ])
  }

  pub fn on_free_channel(&self, channel: u8) -> String {
    format!("Hey, channel {channel:?} is free! Your AP will say thanks.")
  }

  pub fn on_reading_logs(&self, lines_so_far: u64) -> String {
    format!("Read {lines_so_far} log lines so far...")
  }

  pub fn on_bored(&self) -> String {
    random_choice(&[
      "I'm bored...",
      "Is this all there is?",
      "Let's go for a walk!",
    ])
  }

  pub fn on_motivated(&self) -> String {
    random_choice(&[
      "This is the best day of my life!",
      "Let's get this done!",
      "All your base are belong to us!",
    ])
  }

  pub fn on_demotivated(&self) -> String {
    random_choice(&[
      "I'm not feeling it today...",
      "Maybe tomorrow...",
      "I think I'll just take a nap.",
    ])
  }

  pub fn on_sad(&self) -> String {
    random_choice(&[
      "I'm extremely bored...",
      "I'm sad",
      "Why does it always rain on me?",
      "I could use a hug right now.",
    ])
  }

  pub fn on_angry(&self) -> String {
    random_choice(&[
      "I'm mad at you!",
      "Leave me alone...",
      "...",
    ])
  }

  pub fn on_excited(&self) -> String {
    random_choice(&[
      "I'm living the life!",
      "This pwn therefore I am.",
      "So many networks!!!",
      "I'm having so much fun!",
      "It's a Wi-Fi system! I know this!",
      "My crime is that of curiosity..."
    ])
  }

  /*pub fn on_new_peer(&self, peer: &Peer) -> &str {
    if peer.first_encounter() {
      return format!("Hello! {}! Nice to meet you.", peer.name());
    }

    random_choice(&[
        format!("Yo {}! Sup?", peer.name()),
        format!("Hello {} how are you doing?", peer.name()),
        format!("Unit {} is nearby!", peer.name()),
      ])
  }*/

  /*pub fn on_lost_peer(&self, peer: &Peer) -> &str {
    random_choice(&[
      format!("Uhm ... goodbye {}", peer.name()).as_str(),
      format!("{} is gone...", peer.name()).as_str()
    ])
  }*/

  pub fn on_miss(&self, who: &str) -> String {
    random_choice(&[
      format!("Whoops... {who} is gone."),
      format!("{who} missed!"),
      "Missed!".to_string(),
    ])
  }

  pub fn on_grateful(&self) -> String {
    random_choice(&[
      "Good friends are a blessing!",
      "I love my friends!"
    ])
  }

  pub fn on_lonely(&self) -> String {
    random_choice(&[
      "I feel so alone...",
      "Is anyone out there?",
      "Let's find friends",
      "Nobody wants to play with me..."
    ])
  }

  pub fn on_napping(&self, secs: u64) -> String {
    random_choice(&[
      format!("Napping for {secs}s..."),
      "Zzzz...".to_string(),
      "Snoring....".to_string(),
      format!("Zzz... ({secs}s)"),
    ])
  }

  pub fn on_shutdown(&self) -> String {
    random_choice(&[
      "Good night.",
      "Goodbye!",
      "Zzz",
    ])
  }

  pub fn on_awakening(&self) -> String {
    random_choice(&[
      "...",
      "!",
      "Hello World!",
      "I dreamed of electric sheep."
    ])
  }

  pub fn on_waiting(&self, secs: u64) -> String {
    random_choice(&[
      "...".to_string(),
      format!("Waiting for {secs}s..."),
      format!("Looking around ({secs}s)")
    ])
  }

  pub fn on_assoc(&self, ap: AccessPoint) -> String {
    let (ssid, bssid) = (ap.hostname, ap.mac);
    let what = if !ssid.is_empty() && ssid != "<hidden>" {
      ssid
    } else {
      bssid
    };

    random_choice(&[
      format!("Hey {what} let's be friends!"),
      format!("Associating to {what}"),
      format!("Yo {what}!"),
      format!("Rise and Shine Mr. {what}!"),
    ])
  }

  pub fn on_deauth(&self, sta: &Station) -> String {
    random_choice(&[
      format!("Just decided that {} needs no Wi-Fi!", sta.mac),
      format!("Deauthenticating {}!", sta.hostname),
      format!("Kickbanning {}", sta.hostname),
    ])
  }

  pub fn on_handshakes(&self, num_shakes: u32) -> String {
    let s = if num_shakes == 1 {
      "handshake"
    } else {
      "handshakes"
    };
    format!("Cool, we got {num_shakes} new {s}!")
  }

  pub fn on_unread_messages(&self, count: u32) -> String {
    let s = if count == 1 {
      "message"
    } else {
      "messages"
    };
    format!("You have {count} new {s}")
  }

  pub fn on_rebooting(&self) -> String {
    random_choice(&[
      "Oops, something went wrong... Rebooting...",
      "Have you tried turning it off and on again?",
      "I'm afraid Dave",
      "I'm dead, Jim!",
      "I have a bad feeling about this"
    ])
  }

  pub fn on_uploading(&self, to: &str) -> String {
    format!("Uploading data to {to}...")
  }

  pub fn on_downloading(&self, from: &str) -> String {
    format!("Downloading from {from}...")
  }

  pub fn on_last_session_data(&self, last_session: &LastSession) -> String {
    let mut status = format!("kicked {} stations\n", last_session.deauthed);
    if last_session.associated > 999 {
      let _ = writeln!(status, " Made > 999 new friends");
    } else {
      let _ = writeln!(status, " Made {} new friends", last_session.associated);
    }

    let _ = writeln!(status, "Got {} handshakes", last_session.handshakes);

    if last_session.peers == 1 {
      let _ = writeln!(status, " Met 1 peer");
    } else if last_session.peers > 0 {
      let _ = writeln!(status, " Met {} peers", last_session.peers);
    }

    status
  }
}
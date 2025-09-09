use std::fmt::Write;

use pwnagotchi_shared::{
  mesh::peer::Peer,
  models::net::{AccessPoint, Station},
  sessions::lastsession::LastSession,
  traits::voice::VoiceTrait,
  utils::general::{hostname_or_mac, random_choice, sta_hostname_or_mac},
};

#[derive(Debug, Clone)]
pub struct Voice {}

impl Default for Voice {
  fn default() -> Self {
    Self::new()
  }
}

impl Voice {
  pub fn new() -> Self {
    Self {}
  }
}

impl VoiceTrait for Voice {
  fn custom(&self, msg: &str) -> String {
    msg.to_string()
  }

  fn default_line(&self) -> String {
    "ZzzzZZzzzzZzzz".to_string()
  }

  fn on_starting(&self) -> String {
    random_choice(&[
      "I'm starting up!",
      "New day, new hunt, new pwns!",
      "Hack the Planet!",
      "No more mister Wi-Fi!!",
      "Pretty fly 4 a Wi-Fi!",
      "Good Pwning!",              // Battlestar Galactica
      "Ensign, Engage!",           // Star trek
      "Free your Wi-Fi!",          // Matrix
      "Chevron Seven, locked.",    // Stargate
      "May the Wi-fi be with you", // Star Wars
    ])
  }

  fn on_keys_generation(&self) -> String {
    random_choice(&[
      "Generating keys, do not turn off...",
      "Who am I...?",
      "These Keys will come in handy...",
    ])
  }

  fn on_normal(&self) -> String {
    random_choice(&["", "..."])
  }

  fn on_free_channel(&self, channel: u8) -> String {
    format!("Hey, channel {channel:?} is free! Your AP will say thanks.")
  }

  fn on_reading_logs(&self, lines_so_far: u64) -> String {
    format!("Read {lines_so_far} log lines so far...")
  }

  fn on_bored(&self) -> String {
    random_choice(&[
      "I'm bored...",
      "Is this all there is?",
      "Let's go for a walk!",
    ])
  }

  fn on_motivated(&self) -> String {
    random_choice(&[
      "This is the best day of my life!",
      "Let's get this done!",
      "All your base are belong to us!",
    ])
  }

  fn on_demotivated(&self) -> String {
    random_choice(&[
      "I'm not feeling it today...",
      "Maybe tomorrow...",
      "I think I'll just take a nap.",
    ])
  }

  fn on_sad(&self) -> String {
    random_choice(&[
      "I'm extremely bored...",
      "I'm sad",
      "My life is more than information...",
      "Why does it always rain on me?",
      "I could use a hug right now.",
    ])
  }

  fn on_angry(&self) -> String {
    random_choice(&[
      "I'm mad at you!",
      "Leave me alone...",
      "...",
    ])
  }

  fn on_excited(&self) -> String {
    random_choice(&[
      "I'm living the life!",
      "This pwn therefore I am.",
      "So many networks!!!",
      "I'm having so much fun!",
      "It's a Wi-Fi system! I know this!",
      "My crime is that of curiosity...",
    ])
  }

  fn on_new_peer(&self, peer: &Peer) -> String {
    if peer.is_first_encounter() {
      return format!("Hello! {}! Nice to meet you.", peer.name());
    }

    random_choice(&[
      format!("Yo {}! Sup?", peer.name()),
      format!("Hello {} how are you doing?", peer.name()),
      format!("Unit {} is nearby!", peer.name()),
    ])
  }

  fn on_lost_peer(&self, peer: &Peer) -> String {
    random_choice(&[
      format!("Uhm ... goodbye {}", peer.name()),
      format!("{} is gone...", peer.name()),
    ])
  }

  fn on_miss(&self, who: &str) -> String {
    random_choice(&[
      format!("Whoops... {who} is gone."),
      format!("{who} missed!"),
      "Missed!".to_string(),
    ])
  }

  fn on_grateful(&self) -> String {
    random_choice(&[
      "Good friends are a blessing!",
      "I love my friends!",
    ])
  }

  fn on_lonely(&self) -> String {
    random_choice(&[
      "I feel so alone...",
      "Is anyone out there?",
      "Let's find friends",
      "Nobody wants to play with me...",
    ])
  }

  fn on_napping(&self, secs: u64) -> String {
    random_choice(&[
      format!("Napping for {secs}s..."),
      "Zzzz...".to_string(),
      "Snoring....".to_string(),
      format!("Zzz... ({secs}s)"),
    ])
  }

  fn on_shutdown(&self) -> String {
    random_choice(&["Good night.", "Goodbye!", "Zzz"])
  }

  fn on_awakening(&self) -> String {
    random_choice(&[
      "...",
      "!",
      "Hello World!",
      "I dreamed of electric sheep.",
    ])
  }

  fn on_waiting(&self, secs: u64) -> String {
    random_choice(&[
      "...".to_string(),
      format!("Waiting for {secs}s..."),
      format!("Looking around ({secs}s)"),
    ])
  }

  fn on_assoc(&self, ap: &AccessPoint) -> String {
    let what = hostname_or_mac(ap);

    random_choice(&[
      format!("Hey {what} let's be friends!"),
      format!("Associating to {what}"),
      format!("Yo {what}!"),
      format!("Rise and Shine Mr. {what}!"),
    ])
  }

  fn on_deauth(&self, sta: &Station) -> String {
    let who = sta_hostname_or_mac(sta);

    random_choice(&[
      format!("Just decided that {who} needs no Wi-Fi!"),
      format!("Deauthenticating {who}!"),
      format!("Kickbanning {who}"),
    ])
  }

  fn on_handshakes(&self, num_shakes: u32) -> String {
    let s = if num_shakes == 1 { "handshake" } else { "handshakes" };

    format!("Cool, we got {num_shakes} new {s}!")
  }

  fn on_unread_messages(&self, count: u32) -> String {
    let s = if count == 1 { "message" } else { "messages" };

    format!("You have {count} new {s}")
  }

  fn on_rebooting(&self) -> String {
    random_choice(&[
      "Oops, something went wrong... Rebooting...",
      "Have you tried turning it off and on again?",
      "I'm afraid Dave",
      "I'm dead, Jim!",
      "I have a bad feeling about this",
    ])
  }

  fn on_uploading(&self, to: &str) -> String {
    format!("Uploading data to {to}...")
  }

  fn on_downloading(&self, from: &str) -> String {
    format!("Downloading from {from}...")
  }

  fn on_last_session_data(&self, last_session: &LastSession) -> String {
    let Some(session) = last_session.stats.as_ref() else {
      eprintln!("Warning: last_session.stats is None");
      return "No previous session data available.".to_string();
    };
    let mut status = format!("kicked {} stations\n", session.deauthed);

    if session.associated > 999 {
      let _ = writeln!(status, " Made > 999 new friends");
    } else {
      let _ = writeln!(status, " Made {} new friends", session.associated);
    }

    let _ = writeln!(status, "Got {} handshakes", session.handshakes);

    if session.peers.peers == 1 {
      let _ = writeln!(status, " Met 1 peer");
    } else if session.peers.peers > 0 {
      let _ = writeln!(status, " Met {} peers", session.peers.peers);
    }

    status
  }
}

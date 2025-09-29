#![warn(
  clippy::complexity,
  clippy::style,
  clippy::suspicious,
  clippy::pedantic,
  clippy::nursery,
  clippy::cargo
)]
#![deny(clippy::correctness, clippy::perf)]
// I occasionally add functions required but not implemented yet
// Helps with TODOS
#![allow(dead_code, reason = "Occasional placeholders for future implementation")]
// I hate documenting
#![allow(
  clippy::missing_errors_doc,
  clippy::missing_docs_in_private_items,
  reason = "Documentation will be added later as the project matures:tm:"
)]
#![allow(clippy::must_use_candidate)]
// Cant do much about that
#![allow(clippy::multiple_crate_versions)]

pub mod components {
  pub mod manager;
}

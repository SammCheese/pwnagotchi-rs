mod dev;
mod pi;
mod portable;

pub use dev::DevHostnameManager;
pub use pi::PiHostnameManager;
pub use portable::PortableHostnameManager;

pub trait HostnameManager: Send + Sync {
  fn get_hostname(&self) -> Result<String, String>;
  fn set_hostname(&mut self, new: &str) -> Result<(), String>;
}

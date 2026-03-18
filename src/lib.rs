pub mod config;
pub mod error;
pub mod utils;
pub mod web;

#[cfg(feature = "kvm")]
pub mod kvm;

#[cfg(feature = "p2p")]
pub mod p2p;

#[cfg(feature = "webrtc")]
pub mod webrtc;

#[cfg(feature = "cloud_init")]
pub mod cloud_init;

#[cfg(feature = "tun")]
pub mod tunnel;

pub mod dht;

#[cfg(feature = "hardware")]
pub mod hardware;

pub use config::Config;
pub use error::{Error, Result};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "ippi";

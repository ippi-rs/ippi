use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Web server error: {0}")]
    Web(String),
    
    #[cfg(feature = "kvm")]
    #[error("KVM error: {0}")]
    Kvm(String),
    
    #[cfg(feature = "p2p")]
    #[error("P2P error: {0}")]
    P2p(String),
    
    #[cfg(feature = "webrtc")]
    #[error("WebRTC error: {0}")]
    WebRtc(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, Error>;
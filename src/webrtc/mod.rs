use crate::{Config, Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct WebRtcManager {
    config: Arc<Config>,
    state: Arc<RwLock<WebRtcState>>,
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    streams: Arc<RwLock<HashMap<String, StreamInfo>>>,
}

#[derive(Debug, Clone)]
pub struct WebRtcState {
    pub enabled: bool,
    pub initialized: bool,
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<TurnServer>,
    pub video_bitrate: u32,
    pub audio_bitrate: u32,
    pub active_sessions: usize,
    pub active_streams: usize,
}

#[derive(Debug, Clone)]
pub struct TurnServer {
    pub url: String,
    pub username: String,
    pub credential: String,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub peer_id: String,
    pub state: SessionState,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub video_enabled: bool,
    pub audio_enabled: bool,
    pub data_channel_open: bool,
    pub stats: SessionStats,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub video_fps: f32,
    pub audio_bitrate: f32,
    pub video_bitrate: f32,
    pub rtt_ms: f32,
    pub packet_loss: f32,
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub stream_id: String,
    pub session_id: String,
    pub stream_type: StreamType,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub framerate: u32,
    pub bitrate: u32,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Video,
    Audio,
    Data,
}

#[derive(Debug, Clone)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct OfferAnswer {
    pub sdp: String,
    pub sdp_type: SdpType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SdpType {
    Offer,
    Answer,
    Pranswer,
    Rollback,
}

impl WebRtcManager {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let state = WebRtcState {
            enabled: config.webrtc.as_ref().map(|w| w.enabled).unwrap_or(false),
            initialized: false,
            stun_servers: config
                .webrtc
                .as_ref()
                .map(|w| w.stun_servers.clone())
                .unwrap_or_default(),
            turn_servers: config
                .webrtc
                .as_ref()
                .map(|w| w.turn_servers.clone())
                .unwrap_or_default(),
            video_bitrate: config
                .webrtc
                .as_ref()
                .map(|w| w.video_bitrate)
                .unwrap_or(2000),
            audio_bitrate: config
                .webrtc
                .as_ref()
                .map(|w| w.audio_bitrate)
                .unwrap_or(128),
            active_sessions: 0,
            active_streams: 0,
        };

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            streams: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if !state.enabled {
            tracing::info!("WebRTC support is disabled in configuration");
            return Ok(());
        }

        #[cfg(feature = "webrtc")]
        {
            match self.initialize_webrtc().await {
                Ok(_) => {
                    state.initialized = true;
                    tracing::info!(
                        "WebRTC initialized with {} STUN servers, {} TURN servers",
                        state.stun_servers.len(),
                        state.turn_servers.len()
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to initialize WebRTC: {}", e);
                    state.enabled = false;
                    return Err(Error::WebRtc(format!("Failed to initialize WebRTC: {}", e)));
                }
            }
        }

        #[cfg(not(feature = "webrtc"))]
        {
            tracing::warn!("WebRTC feature not enabled at compile time");
            state.enabled = false;
        }

        Ok(())
    }

    pub async fn create_session(&self, peer_id: &str) -> Result<String> {
        let mut state = self.state.write().await;

        if !state.enabled || !state.initialized {
            return Err(Error::WebRtc("WebRTC is not initialized".to_string()));
        }

        let session_id = uuid::Uuid::new_v4().to_string();

        let session = SessionInfo {
            session_id: session_id.clone(),
            peer_id: peer_id.to_string(),
            state: SessionState::New,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            video_enabled: false,
            audio_enabled: false,
            data_channel_open: false,
            stats: SessionStats {
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
                video_fps: 0.0,
                audio_bitrate: 0.0,
                video_bitrate: 0.0,
                rtt_ms: 0.0,
                packet_loss: 0.0,
            },
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        state.active_sessions += 1;

        tracing::info!("Created WebRTC session {} for peer {}", session_id, peer_id);

        Ok(session_id)
    }

    pub async fn create_offer(&self, session_id: &str) -> Result<OfferAnswer> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::WebRtc(format!("Session not found: {}", session_id)))?;

        session.state = SessionState::Connecting;
        session.last_activity = Instant::now();

        #[cfg(feature = "webrtc")]
        {
            let offer = self.create_webrtc_offer(session_id).await?;
            Ok(offer)
        }

        #[cfg(not(feature = "webrtc"))]
        {
            // Simulated offer
            let offer = OfferAnswer {
                sdp: format!(
                    "v=0\r\no=- {} 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\na=msid-semantic: WMS\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 0.0.0.0\r\na=ice-ufrag:{}",
                    uuid::Uuid::new_v4(),
                    uuid::Uuid::new_v4()
                ),
                sdp_type: SdpType::Offer,
            };

            Ok(offer)
        }
    }

    pub async fn set_answer(&self, session_id: &str, answer: OfferAnswer) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::WebRtc(format!("Session not found: {}", session_id)))?;

        if answer.sdp_type != SdpType::Answer {
            return Err(Error::WebRtc("Expected SDP answer".to_string()));
        }

        session.state = SessionState::Connected;
        session.last_activity = Instant::now();

        #[cfg(feature = "webrtc")]
        {
            self.set_webrtc_answer(session_id, answer).await?;
        }

        tracing::info!("Set answer for session {}", session_id);

        Ok(())
    }

    pub async fn add_ice_candidate(&self, session_id: &str, candidate: IceCandidate) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::WebRtc(format!("Session not found: {}", session_id)))?;

        session.last_activity = Instant::now();

        #[cfg(feature = "webrtc")]
        {
            self.add_webrtc_ice_candidate(session_id, candidate).await?;
        }

        tracing::debug!("Added ICE candidate for session {}", session_id);

        Ok(())
    }

    pub async fn create_video_stream(
        &self,
        session_id: &str,
        width: u32,
        height: u32,
        framerate: u32,
    ) -> Result<String> {
        let mut sessions = self.sessions.write().await;
        let mut state = self.state.write().await;
        let mut streams = self.streams.write().await;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::WebRtc(format!("Session not found: {}", session_id)))?;

        if session.state != SessionState::Connected {
            return Err(Error::WebRtc("Session not connected".to_string()));
        }

        let stream_id = uuid::Uuid::new_v4().to_string();

        let stream = StreamInfo {
            stream_id: stream_id.clone(),
            session_id: session_id.to_string(),
            stream_type: StreamType::Video,
            codec: "VP8".to_string(),
            width,
            height,
            framerate,
            bitrate: state.video_bitrate,
            active: true,
        };

        streams.insert(stream_id.clone(), stream);
        session.video_enabled = true;
        state.active_streams += 1;

        tracing::info!(
            "Created video stream {} for session {} ({}x{} @ {}fps)",
            stream_id,
            session_id,
            width,
            height,
            framerate
        );

        Ok(stream_id)
    }

    pub async fn create_audio_stream(&self, session_id: &str) -> Result<String> {
        let mut sessions = self.sessions.write().await;
        let mut state = self.state.write().await;
        let mut streams = self.streams.write().await;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::WebRtc(format!("Session not found: {}", session_id)))?;

        if session.state != SessionState::Connected {
            return Err(Error::WebRtc("Session not connected".to_string()));
        }

        let stream_id = uuid::Uuid::new_v4().to_string();

        let stream = StreamInfo {
            stream_id: stream_id.clone(),
            session_id: session_id.to_string(),
            stream_type: StreamType::Audio,
            codec: "OPUS".to_string(),
            width: 0,
            height: 0,
            framerate: 0,
            bitrate: state.audio_bitrate,
            active: true,
        };

        streams.insert(stream_id.clone(), stream);
        session.audio_enabled = true;
        state.active_streams += 1;

        tracing::info!(
            "Created audio stream {} for session {}",
            stream_id,
            session_id
        );

        Ok(stream_id)
    }

    pub async fn send_data(&self, session_id: &str, data: &[u8]) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::WebRtc(format!("Session not found: {}", session_id)))?;

        if session.state != SessionState::Connected {
            return Err(Error::WebRtc("Session not connected".to_string()));
        }

        if !session.data_channel_open {
            return Err(Error::WebRtc("Data channel not open".to_string()));
        }

        session.stats.bytes_sent += data.len() as u64;
        session.stats.packets_sent += 1;
        session.last_activity = Instant::now();

        #[cfg(feature = "webrtc")]
        {
            self.send_webrtc_data(session_id, data).await?;
        }

        Ok(())
    }

    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let mut state = self.state.write().await;
        let mut sessions = self.sessions.write().await;
        let mut streams = self.streams.write().await;

        // Remove all streams for this session
        let stream_ids: Vec<String> = streams
            .iter()
            .filter(|(_, stream)| stream.session_id == session_id)
            .map(|(id, _)| id.clone())
            .collect();

        for stream_id in stream_ids {
            streams.remove(&stream_id);
            state.active_streams = state.active_streams.saturating_sub(1);
        }

        // Remove session
        if sessions.remove(session_id).is_some() {
            state.active_sessions = state.active_sessions.saturating_sub(1);
            tracing::info!("Closed WebRTC session {}", session_id);
        }

        Ok(())
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionInfo>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    pub async fn get_stats(&self) -> Result<WebRtcState> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    pub async fn update_session_stats(&self, session_id: &str, stats: SessionStats) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| Error::WebRtc(format!("Session not found: {}", session_id)))?;

        session.stats = stats;
        session.last_activity = Instant::now();

        Ok(())
    }
}

#[cfg(feature = "webrtc")]
mod rtc {
    use super::*;

    impl WebRtcManager {
        async fn initialize_webrtc(&self) -> Result<()> {
            // In a real implementation, this would initialize WebRTC
            Ok(())
        }

        async fn create_webrtc_offer(&self, _session_id: &str) -> Result<OfferAnswer> {
            // In a real implementation, this would create a WebRTC offer
            Ok(OfferAnswer {
                sdp: "".to_string(),
                sdp_type: SdpType::Offer,
            })
        }

        async fn set_webrtc_answer(&self, _session_id: &str, _answer: OfferAnswer) -> Result<()> {
            // In a real implementation, this would set the remote answer
            Ok(())
        }

        async fn add_webrtc_ice_candidate(
            &self,
            _session_id: &str,
            _candidate: IceCandidate,
        ) -> Result<()> {
            // In a real implementation, this would add an ICE candidate
            Ok(())
        }

        async fn send_webrtc_data(&self, _session_id: &str, _data: &[u8]) -> Result<()> {
            // In a real implementation, this would send data over data channel
            Ok(())
        }
    }
}

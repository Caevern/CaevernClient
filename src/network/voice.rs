use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use futures_util::{SinkExt, StreamExt};
use opus::{Application, Channels, Decoder, Encoder};
use rtc::{
    interceptor::Registry,
    media::Sample,
    media_stream::MediaStreamTrack,
    rtp_transceiver::rtp_sender::{
        RTCRtpCodec, RTCRtpCodingParameters, RTCRtpEncodingParameters, RtpCodecKind,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    println,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
};
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use webrtc::{
    media_stream::{
        track_local::{TrackLocal, static_sample::TrackLocalStaticSample},
        track_remote::{TrackRemote, TrackRemoteEvent},
    },
    peer_connection::{
        MediaEngine, PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler,
        RTCConfigurationBuilder, RTCIceConnectionState, RTCIceGatheringState, RTCIceServer,
        RTCPeerConnectionIceErrorEvent, RTCPeerConnectionIceEvent, RTCPeerConnectionState,
        RTCSessionDescription, register_default_interceptors,
    },
};

use crate::network::start_microphone::start_microphone;

async fn default_local_ip() -> std::io::Result<std::net::IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .expect("Failed to bind local socket");

    socket
        .connect("1.1.1.1:443")
        .await
        .expect("Failed to connect to 1.1.1.1:443");

    Ok(socket.local_addr().expect("Failed to get local addr").ip())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VoiceSignal {
    Offer { sdp: String },
    Answer { sdp: String },
    IceCandidate { candidate: String },
}

#[derive(Clone)]
struct VoiceHandler {
    ice_tx: Sender<bool>,
    tx: Sender<Vec<f32>>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for VoiceHandler {
    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        println!("MIC ICE GATHERING: {:?}", state);
        if state == RTCIceGatheringState::Complete {
            println!("MIC ICE GATHERING COMPLETE");
            let _ = self.ice_tx.send(true).await;
        }
    }
    async fn on_ice_candidate(&self, event: RTCPeerConnectionIceEvent) {
        println!("MIC ICE candidate: {:?}", event.candidate);
    }
    async fn on_ice_candidate_error(&self, event: RTCPeerConnectionIceErrorEvent) {
        eprintln!(
            "ICE ERROR: code={} url={} text={}",
            event.error_code, event.url, event.error_text
        );
    }

    async fn on_ice_connection_state_change(&self, state: RTCIceConnectionState) {
        println!("MIC CLIENT ICE connection state: {:?}", state);
    }

    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        println!("MIC CLIENT connection state: {:?}", state);
    }

    async fn on_track(&self, track: Arc<dyn TrackRemote>) {
        println!("Received remote audio!");

        let mut decoder =
            Decoder::new(48_000, Channels::Mono).expect("Failed to create Opus decoder");

        let tx = self.tx.clone();

        tokio::spawn(async move {
            while let Some(event) = track.poll().await {
                if let TrackRemoteEvent::OnRtpPacket(packet) = event {
                    let mut pcm = vec![0.0f32; 960];

                    match decoder.decode_float(&packet.payload, &mut pcm, false) {
                        Ok(samples) => {
                            let pcm = pcm[..samples].to_vec();

                            let _ = tx.send(pcm).await;
                        }

                        Err(err) => {
                            eprintln!("Opus decode error: {err}");
                        }
                    }
                }
            }
        });
    }
}

pub fn start_speaker(mut rx: Receiver<Vec<f32>>) -> cpal::Stream {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("No output device found");

    println!("Host: {}", host.id().name());
    for host_id in cpal::available_hosts() {
        println!("{host_id:?}");
    }
    println!("Speaker: {}", device.description().unwrap().name());

    let supported = device
        .default_output_config()
        .expect("Failed to get speaker config");

    println!(
        "Speaker format: {} Hz, {} channels, {:?}",
        supported.sample_rate(),
        supported.channels(),
        supported.sample_format()
    );

    let config: cpal::StreamConfig = supported.clone().into();

    let channels = config.channels as usize;

    let mut buffer = VecDeque::<f32>::new();

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => device
            .build_output_stream(
                config,
                move |output: &mut [f32], _| {
                    while let Ok(samples) = rx.try_recv() {
                        buffer.extend(samples);
                    }

                    for frame in output.chunks_mut(channels) {
                        let sample = buffer.pop_front().unwrap_or(0.0);

                        for channel in frame {
                            *channel = sample;
                        }
                    }
                },
                move |err| {
                    eprintln!("Speaker error: {err}");
                },
                None,
            )
            .expect("Failed to build speaker stream"),

        format => panic!("Unsupported speaker format: {format:?}"),
    };

    stream.play().expect("Failed to start speaker");

    stream
}

pub async fn start_voice_handler(user_id: u32, mic_state_arc: Arc<AtomicBool>) {
    let (mut socket, _) = connect_async("ws://178.128.158.197:5000/ws/voice")
        .await
        .expect("Can't connect");
    println!("Connected to websocket /ws/voice");

    socket
        .send(Message::Text(
            format!("{{\"type\":\"Auth\",\"user_data\":\"{}\"}}", user_id).into(),
        ))
        .await
        .expect("Failed to send voice offer");
    println!("Sent auth request");

    let ssrc = rand::random::<u32>();
    let media_track = MediaStreamTrack::new(
        "game-audio".to_string(),
        "microphone".to_string(),
        "Microphone".to_string(),
        RtpCodecKind::Audio,
        vec![RTCRtpEncodingParameters {
            rtp_coding_parameters: RTCRtpCodingParameters {
                ssrc: Some(ssrc),
                ..Default::default()
            },
            codec: RTCRtpCodec {
                mime_type: "audio/opus".to_string(),
                clock_rate: 48_000,
                channels: 2,
                ..Default::default()
            },
            ..Default::default()
        }],
    );
    let audio_track = Arc::new(
        TrackLocalStaticSample::new(media_track).expect("Failed to create local audio track"),
    );

    let (ice_tx, mut ice_rx) = tokio::sync::mpsc::channel(32);
    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<f32>>(100);

    thread::spawn(move || {
        let _stream = start_speaker(rx);

        std::thread::park();
    });

    let config = RTCConfigurationBuilder::default()
        .with_ice_servers(vec![
            RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            },
            RTCIceServer {
                urls: vec!["turn:178.128.158.197:3478".to_string()],
                username: "caevern".to_string(),
                credential: "81b6c7681a8a2101bb50078d8efd1318f1f91934266da218aa0af869f56b47f9"
                    .to_string(),
                ..Default::default()
            },
        ])
        .build();
    println!("Created RTC configuration");

    let mut media_engine = MediaEngine::default();
    media_engine
        .register_default_codecs()
        .expect("Failed to register default codecs");

    let registry = register_default_interceptors(Registry::new(), &mut media_engine)
        .expect("Failed to register default interceptors");

    let ip = default_local_ip().await.expect("Failed to get local ip");

    println!("WebRTC default interface: {ip}");

    let udp_addr = format!("{ip}:0");

    let pc = PeerConnectionBuilder::new()
        .with_configuration(config)
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .with_handler(Arc::new(VoiceHandler { ice_tx, tx }))
        .with_udp_addrs(vec![udp_addr])
        .build()
        .await
        .expect("Failed to create peer connection builder...");
    println!("Created peer connection");

    pc.add_track(audio_track.clone() as Arc<dyn TrackLocal>)
        .await
        .expect("Failed to add local audio track.");

    let offer = pc
        .create_offer(None)
        .await
        .expect("Failed to create peer connection offer...");
    println!("Sent peer connection offer");

    pc.set_local_description(offer)
        .await
        .expect("Failed to set local description for peer connection :C");
    println!("Set local description");

    let _ = ice_rx.recv().await;

    let description = pc
        .local_description()
        .await
        .expect("Missing local description");

    let signal = VoiceSignal::Offer {
        sdp: description.sdp,
    };

    let text = serde_json::to_string(&signal).expect("Failed to serialize voice offer");

    socket
        .send(Message::Text(text.into()))
        .await
        .expect("Failed to send voice offer");

    thread::spawn(move || {
        let mic_state = mic_state_arc.clone();
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        runtime.block_on(async {
            let mut encoder = Encoder::new(48_000, Channels::Mono, Application::Voip)
                .expect("Failed to create Opus encoder");

            let (_stream, rx, _sample_rate) = start_microphone();
            let mut buffer = Vec::<f32>::new();

            while let Ok(samples) = rx.recv() {
                for stereo in samples.chunks_exact(2) {
                    let mono = (stereo[0] + stereo[1]) * 0.5;
                    buffer.push(mono);
                }

                while buffer.len() >= 960 {
                    let frame: Vec<f32> = buffer.drain(..960).collect();

                    if !mic_state.load(Ordering::Relaxed) {
                        let mut encoded = vec![0u8; 1500];

                        let len = encoder
                            .encode_float(&frame, &mut encoded)
                            .expect("Failed to encode Opus");

                        encoded.truncate(len);

                        audio_track
                            .write_sample(
                                ssrc,
                                111,
                                &Sample {
                                    data: encoded.into(),
                                    duration: Duration::from_millis(20),
                                    ..Default::default()
                                },
                                &[],
                            )
                            .await
                            .expect("Failed to write audio sample");
                    }
                }
            }
        });
    });

    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let signal =
                    serde_json::from_str::<VoiceSignal>(&text).expect("Invalid voice signal");

                match signal {
                    VoiceSignal::Answer { sdp } => {
                        println!("Received server answer");

                        let answer =
                            RTCSessionDescription::answer(sdp).expect("Invalid server SDP answer");

                        pc.set_remote_description(answer)
                            .await
                            .expect("Failed to set remote description");

                        println!("Remote description set!");
                    }

                    VoiceSignal::IceCandidate { candidate } => {
                        println!("Received server ICE candidate: {candidate}");

                        // Add it to pc here once your signaling format
                        // contains enough information to construct RTCIceCandidate.
                    }

                    VoiceSignal::Offer { .. } => {
                        println!("Unexpected offer from server");
                    }
                }
            }

            Ok(Message::Close(_)) => {
                println!("Voice WebSocket closed");
                break;
            }

            Ok(_) => {}

            Err(err) => {
                eprintln!("Voice WebSocket error: {err}");
                break;
            }
        }
    }
}

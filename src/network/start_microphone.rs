use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub fn start_microphone() -> (cpal::Stream, mpsc::Receiver<Vec<f32>>, u32) {
    let host = cpal::default_host();

    let device = host.default_input_device().expect("No microphone found");

    println!("Microphone: {}", device.description().unwrap().name());

    let supported = device
        .default_input_config()
        .expect("Failed to get microphone config");

    let sample_rate = supported.sample_rate();
    let channels = supported.channels();

    println!(
        "Microphone format: {} Hz, {} channels, {:?}",
        sample_rate,
        channels,
        supported.sample_format()
    );

    let config: cpal::StreamConfig = supported.clone().into();

    let (tx, rx) = mpsc::channel::<Vec<f32>>();

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => {
            let tx = tx.clone();

            device
                .build_input_stream(
                    config,
                    move |data: &[f32], _| {
                        println!("Received {} samples", data.len());

                        let _ = tx.send(data.to_vec());
                    },
                    move |err| {
                        eprintln!("Microphone error: {err}");
                    },
                    None,
                )
                .expect("Failed to build microphone stream")
        }

        cpal::SampleFormat::I16 => {
            let tx = tx.clone();

            device
                .build_input_stream(
                    config,
                    move |data: &[i16], _| {
                        println!("Received {} samples", data.len());

                        let samples = data.iter().map(|&x| x as f32 / i16::MAX as f32).collect();

                        let _ = tx.send(samples);
                    },
                    move |err| {
                        eprintln!("Microphone error: {err}");
                    },
                    None,
                )
                .expect("Failed to build microphone stream")
        }

        cpal::SampleFormat::U16 => {
            let tx = tx.clone();

            device
                .build_input_stream(
                    config,
                    move |data: &[u16], _| {
                        println!("Received {} samples", data.len());

                        let samples = data
                            .iter()
                            .map(|&x| (x as f32 / u16::MAX as f32) * 2.0 - 1.0)
                            .collect();

                        let _ = tx.send(samples);
                    },
                    move |err| {
                        eprintln!("Microphone error: {err}");
                    },
                    None,
                )
                .expect("Failed to build microphone stream")
        }

        format => panic!("Unsupported microphone format: {format:?}"),
    };

    stream.play().expect("Failed to start microphone");

    (stream, rx, sample_rate)
}

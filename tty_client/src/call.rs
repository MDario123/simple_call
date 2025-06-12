use std::{
    net::{SocketAddr, UdpSocket},
    thread,
    time::Duration,
};

use cpal::{
    BufferSize, InputCallbackInfo, OutputCallbackInfo, SampleRate, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use magnum_opus::{Bitrate, Encoder};

const FRAME_SIZE: usize = 960 * 3; // 60ms at 48kHz
const BITRATE: Bitrate = Bitrate::Bits(16000); // 16kbps
const SILENCE_THRESHOLD_DBFS: f32 = -30.0; // Threshold for silence in dBFS

/// Calculate the RMS (Root Mean Square) of the samples
fn rms(samples: &[f32]) -> f32 {
    let sum_of_squares: f32 = samples.iter().map(|&x| x * x).sum();
    let mean = sum_of_squares / samples.len() as f32;
    mean.sqrt()
}

fn dbfs(samples: &[f32]) -> f32 {
    let rms_value = rms(samples);
    if rms_value == 0.0 {
        return -100.0; // Return a very low value for silence
    }
    20.0 * rms_value.log10()
}

fn is_silent(samples: &[f32]) -> bool {
    dbfs(samples) < SILENCE_THRESHOLD_DBFS
}

pub fn handle_call(udp_sock: UdpSocket, peer_udp_addr: SocketAddr) {
    udp_sock
        .set_nonblocking(true)
        .expect("Error setting non-blocking");

    let host = cpal::default_host();

    let input_stream = {
        let udp_sock = udp_sock.try_clone().unwrap();
        // Initialize input device
        let input_device = host
            .default_input_device()
            .expect("No input device available.");

        let input_config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(48000),
            buffer_size: BufferSize::Fixed(FRAME_SIZE.try_into().unwrap()),
        };

        // Initialize OPUS Encoder to encode input and send through socket
        let mut encoder = Encoder::new(
            48000,
            magnum_opus::Channels::Mono,
            magnum_opus::Application::Voip,
        )
        .unwrap();
        encoder.set_bitrate(BITRATE).unwrap(); // 16kbps bitrate

        let mut in_buff = [0f32; FRAME_SIZE];
        let mut in_buff_filled = 0;
        let mut buff = [0; 4096];

        input_device
            .build_input_stream(
                &input_config,
                move |mut data: &[f32], _meta: &InputCallbackInfo| {
                    while !data.is_empty() {
                        let to_copy = data.len().min(FRAME_SIZE - in_buff_filled);
                        in_buff[in_buff_filled..in_buff_filled + to_copy]
                            .copy_from_slice(&data[..to_copy]);
                        in_buff_filled += to_copy;
                        data = &data[to_copy..];

                        if in_buff_filled == FRAME_SIZE {
                            in_buff_filled = 0; // Reset buffer after sending

                            if is_silent(&in_buff) {
                                udp_sock.send_to(&[], peer_udp_addr).unwrap();
                            } else {
                                let encoded_size =
                                    encoder.encode_float(&in_buff, &mut buff).unwrap();
                                udp_sock
                                    .send_to(&buff[..encoded_size], peer_udp_addr)
                                    .unwrap();
                            }
                        }
                    }
                },
                |e| {
                    panic!("Error in input stream: {}", e);
                },
                None,
            )
            .unwrap()
    };

    // OUTPUT
    let output_stream = {
        let udp_sock = udp_sock.try_clone().unwrap();
        let output_device = host
            .default_output_device()
            .expect("No output device available.");
        let output_config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(48000),
            buffer_size: BufferSize::Fixed(FRAME_SIZE.try_into().unwrap()),
        };
        let mut decoder = magnum_opus::Decoder::new(48000, magnum_opus::Channels::Mono).unwrap();

        let mut recv_buff = [0; 4096];

        let mut out_buff = [0f32; FRAME_SIZE];
        let mut out_buff_filled_l = 0;
        let mut out_buff_filled_r = 0;

        output_device
            .build_output_stream(
                &output_config,
                move |mut data: &mut [f32], _: &OutputCallbackInfo| {
                    while !data.is_empty() {
                        // Copy all that's possible from out_buff to data
                        let to_copy = data.len().min(out_buff_filled_r - out_buff_filled_l);
                        data[..to_copy].copy_from_slice(
                            &out_buff[out_buff_filled_l..out_buff_filled_l + to_copy],
                        );
                        out_buff_filled_l += to_copy;
                        data = &mut data[to_copy..];

                        if out_buff_filled_l == out_buff_filled_r {
                            if let Ok((size, _)) = udp_sock.recv_from(&mut recv_buff) {
                                println!("Received {} bytes from UDP", size);
                                out_buff_filled_l = 0;
                                out_buff_filled_r = FRAME_SIZE;
                                if size == 0 {
                                    out_buff.fill(0.0); // Fill with silence if no data
                                } else {
                                    decoder
                                        .decode_float(&recv_buff[..size], &mut out_buff, false)
                                        .unwrap();
                                }
                            }
                        }
                    }
                },
                |e| {
                    panic!("Error in output stream: {}", e);
                },
                None,
            )
            .unwrap()
    };

    input_stream.play().expect("Error playing input stream");
    output_stream.play().expect("Error playing output stream");

    // Keep the streams alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

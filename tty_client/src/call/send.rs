use std::net::{SocketAddr, UdpSocket};

use cpal::InputCallbackInfo;
use nnnoiseless::DenoiseState;
use opus::{Bitrate, Encoder};

use super::FRAME_SIZE;

/// Threshold for silence detection in dBFS
const SILENCE_THRESHOLD_DBFS: f32 = -40.0;
/// Gain factor for audio samples
const GAIN: f32 = 2.0;
/// The desired bitrate for the Opus encoder.
const BITRATE: Bitrate = Bitrate::Bits(32_000); // 32kbps

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

fn clean_audio(samples: &mut [f32], denoiser: &mut DenoiseState, denoiser_buff: &mut [f32]) {
    for sample in samples.iter_mut() {
        *sample *= 32768.0 * GAIN; // Scale to i16 range
        *sample = sample.clamp(-32768.0, 32767.0);
    }

    for i in (0..samples.len()).step_by(DenoiseState::FRAME_SIZE) {
        denoiser.process_frame(denoiser_buff, &samples[i..i + DenoiseState::FRAME_SIZE]);

        samples[i..i + DenoiseState::FRAME_SIZE]
            .copy_from_slice(&denoiser_buff[..DenoiseState::FRAME_SIZE]);
    }

    for sample in samples.iter_mut() {
        *sample /= 32768.0;
    }
}

pub(crate) fn create_microphone_callback(
    udp_sock: UdpSocket,
    peer_udp_addr: SocketAddr,
) -> impl FnMut(&[f32], &InputCallbackInfo) {
    // Initialize OPUS Encoder to encode input and send through socket
    let mut encoder = Encoder::new(48000, opus::Channels::Mono, opus::Application::Voip).unwrap();
    encoder.set_bitrate(BITRATE).unwrap();

    let mut denoiser = DenoiseState::new();

    let mut in_buff = [0f32; FRAME_SIZE];
    let mut noise_red_buff = [0f32; DenoiseState::FRAME_SIZE];
    let mut in_buff_filled = 0;
    let mut buff = [0; 4096];

    let mut id: u8 = 0;

    move |mut data: &[f32], _meta: &InputCallbackInfo| {
        while !data.is_empty() {
            let to_copy = data.len().min(FRAME_SIZE - in_buff_filled);
            in_buff[in_buff_filled..in_buff_filled + to_copy].copy_from_slice(&data[..to_copy]);
            in_buff_filled += to_copy;
            data = &data[to_copy..];

            if in_buff_filled == FRAME_SIZE {
                in_buff_filled = 0; // Reset buffer after sending

                // Clean audio samples
                clean_audio(&mut in_buff, &mut denoiser, &mut noise_red_buff);

                if is_silent(&in_buff) {
                    // udp_sock.send_to(&[id], peer_udp_addr).unwrap();
                } else {
                    let encoded_size = encoder.encode_float(&in_buff, &mut buff[1..]).unwrap();

                    buff[0] = id; // Set packet ID

                    udp_sock
                        .send_to(&buff[..encoded_size + 1], peer_udp_addr)
                        .unwrap();
                }

                id = id.wrapping_add(1);
            }
        }
    }
}

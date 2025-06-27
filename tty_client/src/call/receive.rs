use std::{net::UdpSocket, time::Instant};

use cpal::OutputCallbackInfo;

use super::FRAME_SIZE;

fn bytes_human_readable(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f32 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f32 / (1024.0 * 1024.0))
    }
}

fn duration_human_readable(duration: std::time::Duration) -> String {
    format!(
        "{}.{:03} seconds",
        duration.as_secs(),
        duration.subsec_millis()
    )
}

pub(crate) fn create_speaker_callback(
    udp_sock: UdpSocket,
) -> impl FnMut(&mut [f32], &OutputCallbackInfo) {
    udp_sock
        .set_nonblocking(true)
        .expect("Error setting non-blocking");

    let mut decoder = opus::Decoder::new(48000, opus::Channels::Mono).unwrap();

    let mut recv_buff = [0; 4096];

    let mut out_buff = [0f32; FRAME_SIZE];
    let mut out_buff_filled_l = 0;
    let mut out_buff_filled_r = 0;

    let mut bytes_received: usize = 0;

    let start_time = Instant::now();
    let mut last_metrics_time = start_time;

    move |mut data: &mut [f32], _: &OutputCallbackInfo| {
        let elapsed = last_metrics_time.elapsed();
        if elapsed.as_secs() >= 1 {
            last_metrics_time = Instant::now();
            println!(
                "Received: {} in {}.",
                bytes_human_readable(bytes_received),
                duration_human_readable(start_time.elapsed()),
            );
        }
        while !data.is_empty() {
            // Copy all that's possible from out_buff to data
            let to_copy = data.len().min(out_buff_filled_r - out_buff_filled_l);
            data[..to_copy]
                .copy_from_slice(&out_buff[out_buff_filled_l..out_buff_filled_l + to_copy]);
            out_buff_filled_l += to_copy;
            data = &mut data[to_copy..];

            if out_buff_filled_l == out_buff_filled_r {
                let received = udp_sock.recv_from(&mut recv_buff);
                match received {
                    Ok((size, _)) => {
                        bytes_received += size + 24;

                        let decoded = decoder
                            .decode_float(&recv_buff[0..size], &mut out_buff, false)
                            .unwrap();

                        out_buff_filled_l = 0;
                        out_buff_filled_r = decoded;
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("Error receiving data: {}", e);
                        }

                        let decoded = decoder.decode_float(&[], &mut out_buff, false).unwrap();

                        out_buff_filled_l = 0;
                        out_buff_filled_r = decoded;
                    }
                }
            }
        }
    }
}

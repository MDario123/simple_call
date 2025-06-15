use std::net::UdpSocket;

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

pub(crate) fn create_speaker_callback(
    udp_sock: UdpSocket,
) -> impl FnMut(&mut [f32], &OutputCallbackInfo) {
    udp_sock
        .set_nonblocking(true)
        .expect("Error setting non-blocking");

    let mut decoder = opus::Decoder::new(48000, opus::Channels::Mono).unwrap();

    let mut recv_buff = [0; 4096];

    let mut out_buff = [0f32; FRAME_SIZE];
    let mut out_buff_filled_l = FRAME_SIZE;

    let mut last_id: u8 = 0;

    let mut bytes_received: usize = 0;

    let instant = std::time::Instant::now();

    move |mut data: &mut [f32], _: &OutputCallbackInfo| {
        while !data.is_empty() {
            // Copy all that's possible from out_buff to data
            let to_copy = data.len().min(FRAME_SIZE - out_buff_filled_l);
            data[..to_copy]
                .copy_from_slice(&out_buff[out_buff_filled_l..out_buff_filled_l + to_copy]);
            out_buff_filled_l += to_copy;
            data = &mut data[to_copy..];

            if out_buff_filled_l == FRAME_SIZE {
                if let Ok((size, _)) = udp_sock.recv_from(&mut recv_buff) {
                    bytes_received += size + 24;
                    let elapsed = instant.elapsed();
                    println!(
                        "Packet {}. Size: {}. Bytes received: {}. Elapsed: {:.2?} seconds.",
                        recv_buff[0],
                        size,
                        bytes_human_readable(bytes_received),
                        elapsed,
                    );
                    if recv_buff[0] != last_id.wrapping_add(1) {
                        println!(
                            "Packet ID mismatch: expected {}, got {}.",
                            last_id.wrapping_add(1),
                            recv_buff[0]
                        );
                    }
                    last_id = recv_buff[0];
                    out_buff_filled_l = 0;

                    decoder
                        .decode_float(&recv_buff[1..size], &mut out_buff, false)
                        .unwrap();
                } else {
                    let elapsed = instant.elapsed();
                    println!(
                        "Packet {}. Size: {}. Bytes received: {}. Elapsed: {:.2?} seconds.",
                        last_id,
                        0,
                        bytes_human_readable(bytes_received),
                        elapsed,
                    );
                    last_id = last_id.wrapping_add(1);
                    out_buff_filled_l = 0;
                    decoder.decode_float(&[], &mut out_buff, false).unwrap();
                }
            }
        }
    }
}

use std::net::UdpSocket;

use cpal::OutputCallbackInfo;

use super::FRAME_SIZE;

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
                    last_id = last_id.wrapping_add(1);
                    out_buff_filled_l = 0;
                    decoder.decode_float(&[], &mut out_buff, false).unwrap();
                }
            }
        }
    }
}

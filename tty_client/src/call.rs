mod receive;
mod send;

use std::{
    net::{SocketAddr, UdpSocket},
    thread,
    time::Duration,
};

use cpal::{
    BufferSize, SampleRate, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use receive::create_speaker_callback;
use send::create_microphone_callback;

/// Length of a single packet's audio frame in samples.
/// This is 60ms of audio at 48kHz sample rate.
const FRAME_SIZE: usize = 960 * 3;

pub fn handle_call(udp_sock: UdpSocket, peer_udp_addr: SocketAddr) {
    udp_sock
        .set_nonblocking(true)
        .expect("Error setting non-blocking");

    let host = cpal::default_host();

    // INPUT
    let input_stream = {
        // Initialize input device
        let input_device = host
            .default_input_device()
            .expect("No input device available.");

        let input_config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(48000),
            buffer_size: BufferSize::Fixed(FRAME_SIZE.try_into().unwrap()),
        };

        input_device
            .build_input_stream(
                &input_config,
                create_microphone_callback(udp_sock.try_clone().unwrap(), peer_udp_addr),
                |e| {
                    panic!("Error in input stream: {}", e);
                },
                None,
            )
            .unwrap()
    };

    // OUTPUT
    let output_stream = {
        let output_device = host
            .default_output_device()
            .expect("No output device available.");
        let output_config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(48000),
            buffer_size: BufferSize::Default,
        };

        output_device
            .build_output_stream(
                &output_config,
                create_speaker_callback(udp_sock.try_clone().unwrap()),
                |e| {
                    panic!("Error in output stream: {}", e);
                },
                None,
            )
            .unwrap()
    };

    input_stream.play().expect("Error playing input stream");
    thread::sleep(Duration::from_millis(40));
    output_stream.play().expect("Error playing output stream");

    // Keep the streams alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

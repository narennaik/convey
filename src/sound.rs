use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::thread;

pub fn play_start() {
    play(include_bytes!("../assets/beep-start.wav"));
}

pub fn play_stop() {
    play(include_bytes!("../assets/beep-stop.wav"));
}

fn play(bytes: &'static [u8]) {
    // Spawn a thread to play the sound without blocking
    thread::spawn(move || {
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(decoder) = Decoder::new(Cursor::new(bytes)) {
                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    sink.append(decoder);
                    sink.sleep_until_end();
                }
            }
        }
    });
}

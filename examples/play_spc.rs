// Plays an SPC music file via Portaudio

extern crate snes_spc;
extern crate portaudio;

use std::env;
use std::fs::File;
use std::io::{self, Read};

use portaudio as pa;
use snes_spc::{SpcPlayer, Filter};

const FRAME_SIZE: usize = 4096;

fn load_file(filename: &str) -> Result<Box<[u8]>, io::Error> {
    let mut file = try!(File::open(filename));
    let mut contents: Vec<u8> = Vec::new();
    try!(file.read_to_end(&mut contents));
    Ok(contents.into_boxed_slice())
}

fn main() {
    // Create emulator and filter
    let mut spc_player = SpcPlayer::new();
    let mut filter = Filter::new();

    // Load SPC
    let filename = match env::args().nth(1) {
        Some(val) => val,
        None => "test.spc".to_string(),
    };
    let spc = load_file(&filename).expect("Could not read SPC file");

    spc_player.load_spc(&spc).unwrap();
    spc_player.clear_echo();
    filter.clear();

    // Play audio
    let pa = pa::PortAudio::new().unwrap();
    let settings =
        pa.default_output_stream_settings(2, snes_spc::SAMPLE_RATE as f64, FRAME_SIZE as u32)
            .unwrap();
    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        spc_player.play(buffer).unwrap();
        filter.run(buffer);
        pa::Continue
    };
    let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();
    stream.start().unwrap();

    println!("Press enter to quit");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

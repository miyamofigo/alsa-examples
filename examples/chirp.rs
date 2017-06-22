extern crate alsa;

#[macro_use] 
extern crate examples;

use std::ffi::CString;
use std::f32::consts::PI;

use alsa::{ Direction, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM };

const SAMPLING_FREQUENCY: u32 = 44100;

fn chirp(max: f32, min: f32, cur: u32, lim: u32) -> f32 {
    (max + (min - max) * cur as f32 / (lim - 1) as f32 / 2.0) * cur as f32
}

fn main() {
    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();
    prepare_default_pcm!(pcm);
    let (max, min, amp, size) = (
        2500.0, 
        1500.0, 
        0.5,
        SAMPLING_FREQUENCY / 5
    );
    
    let buf: Vec<_> = (0..size).into_iter()
        .map(|i| chirp(max, min, i, size - 1))
        .map(|freq| amp * (2.0 * PI * freq / SAMPLING_FREQUENCY as f32).sin())
        .collect();

    let io = pcm.io_f32().unwrap();
    io.writei(buf.as_slice()).unwrap();
    pcm.drain().unwrap();
}

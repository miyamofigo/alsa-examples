extern crate alsa;

#[macro_use] 
extern crate examples;

use std::ffi::CString;
use std::f32::consts::PI;

use alsa::{ Direction, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM };

const SAMPLING_FREQUENCY: u32 = 44100;
const FREQUENCY: u32 = 500;

fn envelope(max: f32, min: f32, cur: u32, lim: u32) -> f32 {
    max + (min - max) * cur as f32 / (lim - 1) as f32 
}

fn main() {
    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();
    prepare_default_pcm!(pcm);
    let (max, min, size) = (0.5, 0.0, SAMPLING_FREQUENCY * 4);
    
    let buf: Vec<_> = (0..size).into_iter()
        .map(|i| envelope(max, min, i, size - 1))
        .enumerate()
        .map(|(i, a)| a * (2.0 * PI * FREQUENCY as f32 * i as f32 / SAMPLING_FREQUENCY as f32).sin())
        .collect();

    let io = pcm.io_f32().unwrap();
    io.writei(buf.as_slice()).unwrap();
    pcm.drain().unwrap();
}

extern crate alsa;

#[macro_use] 
extern crate examples;

use std::ffi::CString;
use std::f32::consts::PI;
use alsa::{ Direction, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM }; 
const SAMPLING_FREQUENCY: u32 = 44100;
const GAIN: f32 = 0.1;

fn main() {
    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();
    prepare_default_pcm!(pcm);
    let (size, amps, freqs) = (
        SAMPLING_FREQUENCY * 4,
        vec![(1.0, 4.0), (0.8, 2.0), (0.6, 1.0), (0.5, 0.5), (0.4, 0.2)],
        vec![440, 880, 1320, 1760, 2200]
    );
    
    let buf: Vec<_> = (0..size).into_iter()
        .map(|i| amps.iter().enumerate()
            .map(|(i, &(x, y))| x * (-5.0 * i as f32 / (size as f32 * y)).exp()) 
            .zip(freqs.iter())
            .fold(0.0, |acc, (a, &f)| acc + a * (2.0 * PI * f as f32 * i as f32 / size as f32).sin()))
        .map(|x| x * GAIN)
        .enumerate()
        .map(|(i, x)| {
            let fader_mask = size / 100; 
            match i {
                i if (i as u32) < fader_mask => x * i as f32 / fader_mask as f32,
                i if (i as u32) > (size - fader_mask) => x * (size - i as u32) as f32 / fader_mask as f32,
                _ => x
            }
        })
        .collect();

    let io = pcm.io_f32().unwrap();
    io.writei(buf.as_slice()).unwrap();
    pcm.drain().unwrap();
}

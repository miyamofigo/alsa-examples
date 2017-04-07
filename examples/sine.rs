extern crate alsa;
extern crate rayon;

#[macro_use] 
extern crate examples;

use std::ffi::CString;
use std::f32::consts::PI;

use alsa::{ Direction, Output, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM };
use rayon::prelude::*;

const FREQUENCY: u32 = 500;
const SAMPLING_FREQUENCY: u32 = 44100;
const AMPLITUDE: f32 = 1.0;

fn main() {
    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();
    prepare_default_pcm!(pcm);

    println!("PCM status: {:?}, {:?}", pcm.state(), pcm.hw_params_current().unwrap());
    let mut output = Output::buffer_open().unwrap();
    pcm.dump(&mut output).unwrap();
    println!("== PCM dump ==\n{}", output);

    let buf: Vec<_> = (0..SAMPLING_FREQUENCY)
        .into_par_iter()
        .map(|i| {
            let phase = i as f32 * PI * FREQUENCY as f32 * 2.0 / SAMPLING_FREQUENCY as f32;
            AMPLITUDE * phase.sin()
        })
        .collect();

    let io = pcm.io_f32().unwrap();
    
    io.writei(buf.as_slice()).unwrap();
    pcm.drain().unwrap();
}

extern crate alsa;
extern crate rayon;

#[macro_use] 
extern crate examples;

use std::ffi::CString;
use std::f32::consts::PI;

use alsa::{ Direction, Output, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM };
use rayon::prelude::*;

const AMPLITUDE: f32 = 3.0;
const SAMPLING_FREQUENCY: u32 = 44100;

const BUFF_LEN: u32 = SAMPLING_FREQUENCY * 2;
const FADE_LEN: u32 = BUFF_LEN / 100;
const PHASE_LEN: u32 = SAMPLING_FREQUENCY / 4;

fn fade(val: f32, idx: u32) -> f32 {
    val * idx as f32 / FADE_LEN as f32
}

fn main() {
    const OCTAVE: [f32; 8] = [
        261.63, 293.66, 329.63, 349.23,
        392.00, 440.00, 493.88, 523.25
    ];

    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();
    prepare_default_pcm!(pcm);

    println!("PCM status: {:?}, {:?}", pcm.state(), pcm.hw_params_current().unwrap());
    let mut output = Output::buffer_open().unwrap();
    pcm.dump(&mut output).unwrap();
    println!("== PCM dump ==\n{}", output);

    let buf: Vec<_> = (0..BUFF_LEN).into_par_iter().map(|i| {
        let (idx, freq) = ( i % PHASE_LEN, OCTAVE[ (i / PHASE_LEN) as usize ] );
        let phase = idx as f32 * PI * 2.0 * freq / BUFF_LEN as f32;
        match idx {
            idx if idx < FADE_LEN => fade(AMPLITUDE * phase.sin(), idx),
            idx if idx > PHASE_LEN - FADE_LEN - 1 => fade(AMPLITUDE * phase.sin(), PHASE_LEN - idx + 1),
            _ => AMPLITUDE * phase.sin()
        }
    }).collect();

    let io = pcm.io_f32().unwrap();

    io.writei(buf.as_slice()).unwrap();
    pcm.drain().unwrap();
}

extern crate examples;
extern crate rayon;

use examples::{ hann, read_wave_mono16 };
use rayon::prelude::*;
use std::f32::consts::PI;

const SAMPLE_FILE: &str = "examples/resources/sine_500hz.wav";

fn compute_weight(i: i32, j: i32) -> (f32, f32) {
    let arg = 2.0 * PI * i as f32 * j as f32 / 64 as f32;
    (arg.cos(), -arg.sin())
}

fn main() {
    let wave = read_wave_mono16(SAMPLE_FILE);
    let data: Vec<_> = hann(64).into_iter().enumerate().map(|(i, w)| wave.data[i] * w).collect();
    
    let (rs, is): (Vec<_>, Vec<_>) = (
        (0..64).into_par_iter().map(|i| (0..64).into_iter().fold(0.0, |acc, j| {
            let (rw, iw) = compute_weight(i, j);
            acc + rw * data[j as usize] - iw * 0.0
        })).collect()
      , (0..64).into_par_iter().map(|i| (0..64).into_iter().fold(0.0, |acc, j| {
            let (rw, iw) = compute_weight(i, j);
            acc + rw * 0.0 + iw * wave.data[j as usize]
        })).collect()
    );

    for k in 0..64 {
        println!("X({}) = {:.32} + {:.32}i", k, rs[k], is[k]);
    }
}


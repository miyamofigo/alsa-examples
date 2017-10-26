extern crate alsa;
extern crate examples;

use std::ffi::CString;
use alsa::{ Direction, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM }; 
use examples::{ iir_lpf, read_wave_mono16 };

const SAMPLE_FILE: &'static str = "examples/resources/pulse_train.wav";
const INPUT_DELAYERS: usize = 2;
const FILTER_DELAYERS: usize = 2; 

fn get_weight(source: (f32, f32, f32), i: usize) -> f32 {
    match i {
        0 => source.0,
        1 => source.1,
        _ => source.2,
    }
}

fn main() {
    let (sample_frequency, data) = { 
        let wave = read_wave_mono16(SAMPLE_FILE); 
        (wave.format.sample_rate, wave.data)
    };

    let buf: Vec<_> = (0..data.len()).into_iter()
        .map(|i| {
            let base = -5.0 * i as f32 / data.len() as f32;
            10000.0 * base.exp() 
        })
        .enumerate()
        .map(|(i, x)| {
            let (input, filter) = 
                iir_lpf(x / sample_frequency as f32, (2.0_f32).sqrt()); 

            let res = (0..(FILTER_DELAYERS + 1)).into_iter().fold(0.0, |acc, j| match i as isize - j as isize {
                 offset if offset >= 0 => acc + get_weight(input, j) * data[offset as usize],
                 _ =>  acc
            });

            (1..(INPUT_DELAYERS + 1)).into_iter().fold(res, |acc, j| match i as isize - j as isize {
                 offset if offset >= 0 => acc - get_weight(filter, j) * data[offset as usize],
                 _ =>  acc
            })
        })
        .collect();

    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();

    let hw_params = HwParams::any(&pcm).unwrap(); 

    hw_params.set_channels(1).unwrap();     
    hw_params.set_rate(sample_frequency, ValueOr::Nearest).unwrap(); 
    hw_params.set_format(Format::float()).unwrap(); 
    hw_params.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hw_params).unwrap(); 

    let io = pcm.io_f32().unwrap();
    io.writei(buf.as_slice()).unwrap();
    pcm.drain().unwrap();

}

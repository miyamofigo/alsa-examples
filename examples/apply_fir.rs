extern crate alsa;
extern crate examples;

use std::ffi::CString;
use alsa::{ Direction, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM }; 
use examples::{ fir_lpf, hann, read_wave_mono16 };

const SAMPLE_FILE: &str = "examples/resources/sine_500hz_3500hz.wav";

fn main() {
    let (s_fr, data) = { 
        let wave = read_wave_mono16(SAMPLE_FILE); 
        (wave.format.sample_rate, wave.data)
    };
    let (e_fr, delta) = (1000.0 / s_fr as f32, 1000.0 / s_fr as f32);
    let delayers = match (3.1 / delta + 0.5) as isize - 1 {
        j if j % 2 == 0 => j + 1,
        j => j
    };

    let filter = fir_lpf(e_fr, delayers, hann((delayers + 1) as usize)); 

    let buf: Vec<_> = (0..data.len()).into_iter().map(|i| {
        let res: f32 = filter.iter().enumerate().map(|(j, x)| {
            match i >= j {
              true => x * data[i - j],
              _ => 0.0
            }
        }).sum();
        res
    }).collect();

    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();

    let hw_params = HwParams::any(&pcm).unwrap(); 
    hw_params.set_channels(1).unwrap();     
    hw_params.set_rate(s_fr, ValueOr::Nearest).unwrap(); 
    hw_params.set_format(Format::float()).unwrap(); 
    hw_params.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hw_params).unwrap(); 

    let io = pcm.io_f32().unwrap();
    io.writei(buf.as_slice()).unwrap();
    pcm.drain().unwrap();
}


extern crate alsa;
extern crate examples;

use std::ffi::CString;
use alsa::{ Direction, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM }; 
use examples::{ Trigram, iir_lpf, read_wave_mono16 };

const SAMPLE_FILE: &str = "examples/resources/sine_500hz_3500hz.wav";

fn main(){
    let (sample_freq, data) = { 
        let wave = read_wave_mono16(SAMPLE_FILE); 
        (wave.format.sample_rate, wave.data)
    };

    let (cutoff_freq, q_factor) = (
        1000.0 / sample_freq as f32,
        1.0 / (2.0f32).sqrt(),
    );

    let ((d_params, n_params), mut dest, pad) = (
        iir_lpf(cutoff_freq, q_factor),
        vec![0.0; data.len()],
        &0.0f32
    );
    
    for (i, (a, b, c)) in data.iter().trigrams(pad).enumerate() {
        dest[i] += n_params.0 * c;
        dest[i] += n_params.1 * b;
        dest[i] += n_params.2 * a;
        if i == 0 {
            continue;
        } else if i == 1 {
            dest[i] -= d_params.1 * dest[i];
        } else {
            dest[i] -= d_params.1 * dest[i-1];
            dest[i] -= d_params.2 * dest[i-2];
        }
    }

    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();

    let hw_params = HwParams::any(&pcm).unwrap(); 
    hw_params.set_channels(1).unwrap();     
    hw_params.set_rate(sample_freq, ValueOr::Nearest).unwrap(); 
    hw_params.set_format(Format::float()).unwrap(); 
    hw_params.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hw_params).unwrap(); 

    let io = pcm.io_f32().unwrap();
    io.writei(dest.as_slice()).unwrap();
    pcm.drain().unwrap();
}

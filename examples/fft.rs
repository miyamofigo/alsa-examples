extern crate examples;
use examples::{ fft, read_wave_mono16 };

const SAMPLE_FILE: &str = "examples/resources/sine_500hz.wav";

fn main() {
    let wave = read_wave_mono16(SAMPLE_FILE);

    let mut data = wave.data.clone(); 
    data.truncate(64);

    for (idx, &(r, i)) in fft(data).iter().enumerate() {
        println!("X({}) = {:.32} + {:.32}i", idx, r, i);
    }
}

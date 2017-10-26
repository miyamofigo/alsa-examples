extern crate alsa;
extern crate examples;

use std::ffi::CString;
use alsa::{ Direction, ValueOr };
use alsa::pcm::{ Access, Format, HwParams, PCM }; 
use examples::{ fft, fir_lpf, hann, ifft, read_wave_mono16 };

const SAMPLE_FILE: &str = "examples/resources/sine_500hz_3500hz.wav";
const FRAME_LEN: usize = 128;
const DFT_LEN: usize = 256; 

fn build_input(source: &Vec<f32>, i: usize, l: usize, n: usize) -> Vec<(f32, f32)> {
    let (mut frame, mut zeros): (Vec<_>, _) = (
        (0..l).into_iter().map(|j| source[l * i  + j]).collect(),
        vec![0.0; n-l]
    );
    frame.append(&mut zeros);
    fft(frame)
}

fn build_filter(source: &Vec<f32>, l: usize, n: usize) -> Vec<(f32, f32)> {
    let filter: Vec<_> = (0..n).into_iter().map(|i| match i {
        i if i <= l => source[i],
        _ => 0.0
    }).collect(); fft(filter)
}

fn apply_filter(input: Vec<(f32, f32)>, filter: &Vec<(f32, f32)>) -> Vec<f32> {
    let output: Vec<_> = input.into_iter().zip(filter.into_iter()).map(|((x_real, x_image), &(b_real, b_image))| (
        x_real * b_real - x_image * b_image,
        x_image * b_real + x_real * b_image
    )).collect(); 
    ifft(output).into_iter().map(|(r, _)| r).collect()  
}

fn main() {
    let (
        sample_freq, 
        data, 
        edge_freq,
        delta
    ) = { 
        let wave = read_wave_mono16(SAMPLE_FILE); 
        let (rate, data) = (wave.format.sample_rate, wave.data);

        (
            rate, 
            data, 
            1000.0 / rate as f32,
            1000.0 / rate as f32
        )
    };

    let num = match (3.1 / delta + 0.5) as i32 - 1 {
        num if num % 2 == 1 => num + 1,
        num => num
    };

    let (
        fir_filter, 
        frame_num,
        data_length
    ) = ( 
        fir_lpf(edge_freq, num as isize, hann((num + 1) as usize)),
        data.len() / FRAME_LEN,
        data.len()
    );

    let filter = build_filter(&fir_filter, num as usize, DFT_LEN); 

    let buf = (0..frame_num).into_iter().map(|i| {
        let input = build_input(&data, i, FRAME_LEN, DFT_LEN);
        apply_filter(input, &filter)
    }).enumerate().fold(vec![0.0f32; data_length], |mut acc, (i, v)| {
       for (j, &x) in v.iter().enumerate() {
           let offset = i * FRAME_LEN + j;

           if offset < data_length {
               acc[offset] += x;
           }
       }
       acc
    });

    println!("{:?}", buf);

    let pcm = PCM::open(&*CString::new("default").unwrap(), Direction::Playback, false).unwrap();

    let hw_params = HwParams::any(&pcm).unwrap(); 
    hw_params.set_channels(1).unwrap();     
    hw_params.set_rate(sample_freq, ValueOr::Nearest).unwrap(); 
    hw_params.set_format(Format::float()).unwrap(); 
    hw_params.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hw_params).unwrap(); 

    let io = pcm.io_f32().unwrap();
    io.writei(buf.as_slice()).unwrap();
    pcm.drain().unwrap();
}

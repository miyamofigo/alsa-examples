extern crate byteorder;
extern crate rayon;
use std::fs::File; use std::f32::consts::PI;
use std::io::prelude::*;
use std::io::Cursor;

use byteorder::{ LittleEndian, ReadBytesExt }; use rayon::prelude::*;

#[macro_export]
macro_rules! prepare_default_pcm {
    ($pcm:ident) => {
        { 
            let hw_params = HwParams::any(&$pcm).unwrap(); 
            hw_params.set_channels(1).unwrap();     
            hw_params.set_rate(SAMPLING_FREQUENCY, ValueOr::Nearest).unwrap(); 
            hw_params.set_format(Format::float()).unwrap(); 
            hw_params.set_access(Access::RWInterleaved).unwrap();
            $pcm.hw_params(&hw_params).unwrap(); 
        }
    }
} 

macro_rules! genbufs {
    ( $t:ty, $($size:expr),+ ) => { ( $(vec![0 as $t; $size],)+ ) } } const VALIDATION_ERR: &'static str = "an invalid sized vector exists.";

trait Validator {
    fn validate(&self) -> Result<(), &str>;
}

macro_rules! __item {
    ( $i:item ) => ($i) }

macro_rules! s {
    ( $( $( #[$attr:meta] )* pub struct $i:ident { $( $f:ident : $t:ty ),+ } )+ ) =>  {$(
        __item! { $( #[$attr] )*
            pub struct $i { $(pub $f : $t),+ }
        }
        impl $i {
            fn new( $( $f : $t ),+ ) -> Self { $i { $( $f ),+ } }
            fn with_valid( $( $f : $t ),+ ) -> Self { __with_valid!( $( $f ),+ ) }
        }
    )+}
}

macro_rules! __read_file {
    ( $f:expr, $( ($i:ident, $size:expr) ),+ ) => {
        let ( $( mut $i, )+ ) = genbufs!(u8, $( $size ),+);
        $( $f.read_exact(&mut $i).unwrap(); )+
    }
}

macro_rules! __with_valid {
    ( $( $f:ident ),+ ) => {{
        let this = Self::new( $( $f ),+ );
        this.validate().and(Ok(this.clone())).unwrap()
    }}
}

s! {
    #[derive(Clone)]
    pub struct Riff {
        id: Vec<u8>,
        size: u32,
        file_format: Vec<u8>
    }

    #[derive(Clone)]
    pub struct SubcHeader {
        id: Vec<u8>,
        size: u32
    }
}

impl Validator for Riff {
    fn validate(&self) -> Result<(), &str> {
        match (self.id.len(), self.file_format.len()) {
            (4, 4) => Ok(()),
            _ => Err(VALIDATION_ERR)
        }
    }
}

impl Validator for SubcHeader {
    fn validate(&self) -> Result<(), &str> {
        match self.id.len() {
            4 => Ok(()),
            _ => Err(VALIDATION_ERR)
        }
    }
}

trait FromFile {
    fn from_file(file: &mut File) -> Self;
}

impl FromFile for Riff {
    fn from_file(file: &mut File) -> Self {
        __read_file!(file, (_id, 4), (_size, 4), (_ftype, 4));
        Self::with_valid(_id.clone(), u8vec_to_u32_le(_size), _ftype.clone())
    }
}

macro_rules! __from_file {
    ( $file:expr, $( $target:ident ),+ ) => {( $( $target::from_file($file), )+ )}
}

impl FromFile for SubcHeader {
    fn from_file(file: &mut File) -> Self {
        __read_file!(file, (_id, 4), (_size, 4));
        Self::with_valid(_id.clone(), u8vec_to_u32_le(_size))
    }
}

#[derive(Clone)]
pub struct Format {
    pub format: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub bit_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16
}

impl Format {
    fn new(format: u16, channels: u16, sample_rate: u32, bit_rate: u32,
        block_align: u16, bits_per_sample: u16) -> Self {
        Format {
            format, channels, sample_rate, bit_rate, block_align, 
            bits_per_sample
        }
    }
}

impl FromFile for Format {
    fn from_file(file: &mut File) -> Self {
        __read_file!(file, (_format, 2), 
            (_channels, 2),
            (_sample_rate, 4),
            (_bit_rate, 4),
            (_block_align, 2),
            (_bits_per_sample, 2));

        Format::new(u8vec_to_u16_le(_format), 
            u8vec_to_u16_le(_channels), 
            u8vec_to_u32_le(_sample_rate), 
            u8vec_to_u32_le(_bit_rate), 
            u8vec_to_u16_le(_block_align), 
            u8vec_to_u16_le(_bits_per_sample))        
    }
}

fn u8vec_to_u32_le(src: Vec<u8>) -> u32 {
    let mut reader = Cursor::new(src);
    reader.read_u32::<LittleEndian>().unwrap() 
}

fn u8vec_to_u16_le(src: Vec<u8>) -> u16 {
    let mut reader = Cursor::new(src);
    reader.read_u16::<LittleEndian>().unwrap() 
}

unsafe fn from_bytes<'a, T>(buf: &'a [u8]) -> &'a [T] {
    std::slice::from_raw_parts(buf.as_ptr() as *const T, buf.len() / std::mem::size_of::<T>()) 
}

type FormatHeader = SubcHeader;
type DataHeader = SubcHeader;

pub struct Wave {
    pub riff: Riff, 
    pub format_header: FormatHeader,
    pub format: Format,
    pub data_header: DataHeader,
    pub data: Vec<f32>
}

impl Wave {
    pub fn new(riff: Riff, 
        format_header: FormatHeader,
        format: Format,
        data_header: DataHeader,
        data: Vec<f32>) -> Self {
        Wave { 
            riff, 
            format_header, 
            format, 
            data_header, 
            data 
        }
    }

}

pub fn read_wave_mono16(fname: &str) -> Wave {
    let mut file = File::open(fname).unwrap();
    let (riff, format_header, format, data_header) = __from_file!(&mut file, Riff, SubcHeader, Format, SubcHeader);

    __read_file!(&mut file, (tmp, data_header.size as usize));
    let data: Vec<_> = unsafe { 
        from_bytes::<u16>(tmp.as_slice()).into_par_iter().map(|&c| c as f32 / 32768.0).collect()
    };

    Wave::new(
        riff, 
        format_header, 
        format, 
        data_header, 
        data
    )
}

pub fn hann(n: usize) -> Vec<f32> {
    (0..n).into_iter().map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * match i {
        i if i % 2 == 0 => i as f32,
        _ => i as f32 + 0.5
    } / n as f32).cos()).collect() 
}

fn count_stage(n: usize) -> usize { (n as f32).log(2.0) as usize }

fn compute_stage(src: Vec<(f32, f32)>, curr: usize, limit: usize) -> Vec<(f32, f32)> {
    match curr {
        curr if curr >= limit => src,
        _ => compute_stage(match curr - 1 {
            0 => src,
            num => (0..(2usize.pow(num as u32))).into_iter().fold(Vec::new(), |mut acc, i| {
                let res: Vec<_> = (0..(2usize.pow((limit - curr) as u32))).into_iter().map(|j| {
                    let m = 2usize.pow((limit - curr + 1) as u32) * i + j;
                    let (n, r) = (2usize.pow((limit - curr) as u32) + m, 2usize.pow((curr - 1) as u32) * j);

                    let w = 2.0 * PI * r as f32 / n as f32; let ((a_real, a_img), (b_real, b_img), (c_real, c_img)) = (src[m], src[n], (w.cos(), -w.sin()));

                    if curr == limit {
                        (
                            (a_real + b_real, a_img + b_img), 
                            (a_real - b_real, a_img - b_img)
                        )
                    } else { 
                        (
                            (a_real + b_real, a_img + b_img),
                            (
                                (a_real - b_real) * c_real  - (a_img - b_img) * c_img,
                                (a_img - b_img) * c_real  - (a_real - b_real) * c_img
                            )
                        )
                    }
                }).collect();

                let (mut next, mut front, mut back) = (Vec::new(), 
                    res.clone().into_iter().map(|(i, _)| i).collect::<Vec<_>>(), 
                    res.into_iter().map(|(_, j)| j).collect::<Vec<_>>()
                );

                next.append(&mut front); next.append(&mut back);
                acc.append(&mut next); acc
            })
        }, curr + 1, limit)
    }
}

fn indices(len: usize) -> Vec<usize> {
    (0..len).into_iter().map(compute_index_weight).collect()
}

fn compute_index_weight(idx: usize) -> usize {
    let num = match idx {
        0 => 0,
        _ => (idx as f32).log(2.0) as usize
    };
    (0..num).into_iter().fold(0, |acc, i| acc + 2usize.pow(i as u32))
}

fn sort_fft_vec<'a>(v: &'a mut Vec<(f32, f32)>) {
    for (i, j) in indices(v.len()).iter().enumerate().filter(|&(i, j)| i < *j) { 
        v.swap(i, *j); 
    }
}
 
pub fn fft(src: Vec<f32>) -> Vec<(f32, f32)> {  
    let (stage_num, pair_v) = (
        count_stage(src.len()),
        src.into_iter().map(|i| (i, 0.0)).collect()
    );

    let mut res = compute_stage(pair_v, 1, stage_num);
    sort_fft_vec(&mut res); res
}

fn sinc(x: f32) -> f32 {
    match x {
        0.0 => 1.0,
        _ => x.sin() / x
    }
}

pub fn fir_lpf(freq: f32, num: isize, src: Vec<f32>) -> Vec<f32> {
    (0..(num + 1)).into_iter().map(|i| 2.0 * freq * sinc(2.0 * PI * freq * (i - num / 2) as f32))
        .zip(src.iter()).map(|(b, w)| b * w).collect()
}

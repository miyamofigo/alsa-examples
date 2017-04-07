#[macro_export]
macro_rules! prepare_default_pcm {
    ($pcm:ident) => {{
        let hw_params = HwParams::any(&$pcm).unwrap();
        hw_params.set_channels(1).unwrap();     
        hw_params.set_rate(SAMPLING_FREQUENCY, ValueOr::Nearest).unwrap();
        hw_params.set_format(Format::float()).unwrap();
        hw_params.set_access(Access::RWInterleaved).unwrap();
        $pcm.hw_params(&hw_params).unwrap(); 
    }}
}

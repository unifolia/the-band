pub fn clamp_unit(sample: f32) -> f32 {
    if !sample.is_finite() {
        0.0
    } else {
        sample.clamp(-1.0, 1.0)
    }
}

pub fn soft_limit(sample: f32) -> f32 {
    if !sample.is_finite() {
        return 0.0;
    }
    (sample * 1.35).tanh() * 0.92
}

pub fn equal_power_pan(mono: f32, pan: f32) -> (f32, f32) {
    let pan = pan.clamp(-1.0, 1.0);
    let angle = (pan + 1.0) * core::f32::consts::FRAC_PI_4;
    (mono * angle.cos(), mono * angle.sin())
}

pub fn seconds_to_samples(seconds: f32, sample_rate: f32) -> usize {
    (seconds.max(0.0) * sample_rate.max(1.0)).round() as usize
}

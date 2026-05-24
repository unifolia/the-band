use crate::instruments::InstrumentBank;
use crate::percussion::PercussionVoiceKind;

const TAU: f32 = core::f32::consts::PI * 2.0;

pub fn render_instrument(
    bank: InstrumentBank,
    frequency: f32,
    t: f32,
    duration: f32,
    velocity: f32,
    event_seed: u64,
    sample_index: u64,
) -> f32 {
    match bank {
        InstrumentBank::Harp => harp(frequency, t, velocity, event_seed, sample_index),
        InstrumentBank::Bass => bass(frequency, t, duration, velocity),
        InstrumentBank::Pads => pads(frequency, t, duration, velocity, event_seed),
        InstrumentBank::Synth => synth_lead(frequency, t, duration, velocity, event_seed),
        InstrumentBank::Clarinet => {
            clarinet(frequency, t, duration, velocity, event_seed, sample_index)
        }
    }
}

pub fn render_percussion(
    kind: PercussionVoiceKind,
    t: f32,
    velocity: f32,
    event_seed: u64,
    sample_index: u64,
) -> f32 {
    match kind {
        PercussionVoiceKind::Kick => kick(t, velocity, event_seed, sample_index),
        PercussionVoiceKind::Snare => snare(t, velocity, event_seed, sample_index),
        PercussionVoiceKind::Hat => hat(t, velocity, event_seed, sample_index),
        PercussionVoiceKind::Tom => tom(t, velocity, 96.0, event_seed),
        PercussionVoiceKind::Woodblock => woody(t, velocity, 720.0, 0.055, event_seed),
        PercussionVoiceKind::Clave => woody(t, velocity, 1180.0, 0.04, event_seed),
        PercussionVoiceKind::LogDrum => woody(t, velocity, 210.0, 0.16, event_seed),
        PercussionVoiceKind::Shaker => shaker(t, velocity, event_seed, sample_index),
        PercussionVoiceKind::CanKick => can_kick(t, velocity, event_seed, sample_index),
        PercussionVoiceKind::CanSnare => can_snare(t, velocity, event_seed, sample_index),
        PercussionVoiceKind::Scrape => scrape(t, velocity, event_seed, sample_index) * 0.8,
        PercussionVoiceKind::AltScrape => alt_scrape(t, velocity, event_seed, sample_index),
        PercussionVoiceKind::Bayan => tabla_low(t, velocity, event_seed),
        PercussionVoiceKind::Dayan => tabla_tuned(t, velocity, 270.0, event_seed),
        PercussionVoiceKind::NaTin => tabla_tuned(t, velocity, 820.0, event_seed) * 0.7,
        PercussionVoiceKind::MutedTap => woody(t, velocity, 430.0, 0.035, event_seed) * 0.8,
    }
}

pub fn instrument_tail_seconds(bank: InstrumentBank) -> f32 {
    match bank {
        InstrumentBank::Harp => 0.7,
        InstrumentBank::Bass => 0.24,
        InstrumentBank::Pads => 0.95,
        InstrumentBank::Synth => 0.32,
        InstrumentBank::Clarinet => 0.28,
    }
}

pub fn percussion_tail_seconds(kind: PercussionVoiceKind) -> f32 {
    match kind {
        PercussionVoiceKind::Kick | PercussionVoiceKind::CanKick | PercussionVoiceKind::Bayan => {
            0.52
        }
        PercussionVoiceKind::Snare
        | PercussionVoiceKind::CanSnare
        | PercussionVoiceKind::Scrape
        | PercussionVoiceKind::AltScrape => 0.28,
        PercussionVoiceKind::Hat
        | PercussionVoiceKind::Shaker
        | PercussionVoiceKind::NaTin
        | PercussionVoiceKind::MutedTap => 0.13,
        PercussionVoiceKind::Tom | PercussionVoiceKind::LogDrum | PercussionVoiceKind::Dayan => {
            0.34
        }
        PercussionVoiceKind::Woodblock | PercussionVoiceKind::Clave => 0.22,
    }
}

fn harp(frequency: f32, t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let excitation = smooth_noise(seed, sample_index, 3) * (-t * 45.0).exp();
    let pluck = triangle(frequency * (1.0 + 0.0015 * seeded_offset(seed)), t);
    let overtone = sine(frequency * 2.01, t) * 0.25 + sine(frequency * 3.0, t) * 0.11;
    let damping = (-t * (2.6 + frequency * 0.002)).exp();
    (pluck * 0.75 + overtone + excitation * 0.16) * damping * velocity * 0.76
}

fn bass(frequency: f32, t: f32, duration: f32, velocity: f32) -> f32 {
    let env = adsr(t, duration, 0.016, 0.08, 0.72, 0.09);
    let tone =
        sine(frequency, t) * 0.72 + triangle(frequency * 0.5, t) * 0.22 + saw(frequency, t) * 0.08;
    (tone * 1.25).tanh() * env * velocity * 0.78
}

fn pads(frequency: f32, t: f32, duration: f32, velocity: f32, seed: u64) -> f32 {
    let wobble = 1.0 + 0.003 * sine(0.18 + seeded_offset(seed).abs() * 0.12, t);
    let env = adsr(t, duration, 0.18, 0.28, 0.72, 0.55);
    let a = sine(frequency * wobble * 0.997, t);
    let b = sine(frequency * 1.502, t) * 0.34;
    let c = triangle(frequency * 2.003, t) * 0.13;
    (a * 0.62 + b + c) * env * velocity * 0.42
}

fn synth_lead(frequency: f32, t: f32, duration: f32, velocity: f32, seed: u64) -> f32 {
    let detune = 1.0 + 0.0025 * seeded_offset(seed);
    let env = adsr(t, duration, 0.008, 0.11, 0.42, 0.14);
    let pulse_width = 0.5 + 0.08 * sine(0.31, t + seeded_offset(seed).abs());
    let raw = saw(frequency * detune, t) * 0.48
        + square_pw(frequency * 0.999, t, pulse_width) * 0.32
        + sine(frequency * 2.0, t) * 0.1;
    let crushed = (raw * 36.0).round() / 36.0;
    (crushed * 0.65 + raw * 0.35) * env * velocity * 0.52
}

fn clarinet(
    frequency: f32,
    t: f32,
    duration: f32,
    velocity: f32,
    seed: u64,
    sample_index: u64,
) -> f32 {
    let vibrato_depth = if t > 0.12 { 0.00024 } else { 0.0 };
    let vibrato = 1.0 + vibrato_depth * sine(5.1 + seeded_offset(seed) * 0.15, t);
    let f = frequency * vibrato;
    let env = adsr(t, duration, 0.035, 0.1, 0.82, 0.16);
    let odd = sine(f, t) * 0.82 + sine(f * 3.0, t) * 0.24 + sine(f * 5.0, t) * 0.08;
    let breath = smooth_noise(seed ^ 0xCAFE, sample_index, 5) * 0.035;
    (odd + breath) * env * velocity * 0.5
}

fn kick(t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let pitch = 44.0 + 92.0 * (-t * 30.0).exp();
    let body = sine(pitch, t) * (-t * 7.8).exp();
    let transient = smooth_noise(seed, sample_index, 2) * (-t * 85.0).exp() * 0.18;
    (body + transient) * velocity * 0.92
}

fn can_kick(t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let pitch = 34.0 + 68.0 * (-t * 24.0).exp();
    let body = (sine(pitch, t) * 0.82 + triangle(pitch * 0.5, t) * 0.24) * (-t * 6.4).exp();
    let knock = sine(82.0 + seeded_offset(seed) * 5.0, t) * (-t * 24.0).exp() * 0.18;
    let transient = smooth_noise(seed ^ 0x7A11, sample_index, 4) * (-t * 52.0).exp() * 0.09;
    (body + knock + transient) * velocity * 0.96
}

fn snare(t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let noise = smooth_noise(seed ^ 0x51A4, sample_index, 4) * (-t * 18.0).exp();
    let body = sine(185.0, t) * (-t * 13.0).exp() * 0.32;
    (noise * 0.54 + body) * velocity * 0.58
}

fn can_snare(t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let noise = smooth_noise(seed ^ 0x51A4, sample_index, 3) * (-t * 19.0).exp();
    let body = sine(172.0 + seeded_offset(seed) * 8.0, t) * (-t * 12.0).exp() * 0.25;
    let ring = (sine(420.0 + seeded_offset(seed ^ 0xC011) * 18.0, t) * 0.22
        + sine(735.0 + seeded_offset(seed ^ 0x5A7E) * 24.0, t) * 0.12)
        * (-t * 15.0).exp();
    (noise * 0.46 + body + ring) * velocity * 0.54
}

fn hat(t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let bright = noise(seed, sample_index) - smooth_noise(seed, sample_index, 9);
    bright * (-t * 68.0).exp() * velocity * 0.32
}

fn tom(t: f32, velocity: f32, base_frequency: f32, seed: u64) -> f32 {
    let bend = base_frequency * (1.0 + 0.42 * (-t * 18.0).exp());
    let tone = sine(bend, t) + triangle(bend * 0.5, t) * 0.25;
    tone * (-t * 8.5).exp() * velocity * (0.72 + seeded_offset(seed).abs() * 0.08)
}

fn woody(t: f32, velocity: f32, frequency: f32, decay: f32, seed: u64) -> f32 {
    let click = sine(frequency * (1.0 + seeded_offset(seed) * 0.02), t) * (-t / decay).exp();
    let knock = triangle(frequency * 0.48, t) * (-t / (decay * 1.8)).exp() * 0.26;
    (click + knock) * velocity * 0.62
}

fn shaker(t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let grain = noise(seed ^ 0xA11E, sample_index) - smooth_noise(seed, sample_index, 6);
    grain * (-t * 26.0).exp() * velocity * 0.28
}

fn scrape(t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let comb = (noise(seed, sample_index) - noise(seed ^ 0x9933, sample_index / 2)) * 0.5;
    let flutter = 0.6 + 0.4 * sine(18.0, t).abs();
    comb * flutter * (-t * 7.0).exp() * velocity * 0.32
}

fn alt_scrape(t: f32, velocity: f32, seed: u64, sample_index: u64) -> f32 {
    let grain =
        smooth_noise(seed ^ 0xA77A, sample_index, 2) - smooth_noise(seed ^ 0x533D, sample_index, 9);
    let rasp =
        noise(seed ^ 0x1CE5, sample_index / 3) - smooth_noise(seed ^ 0x7229, sample_index, 11);
    let drag = 0.48 + 0.52 * sine(11.0 + seeded_offset(seed).abs() * 2.0, t).abs();
    let ring = (sine(310.0 + seeded_offset(seed ^ 0xBEEF) * 14.0, t) * 0.12
        + sine(465.0 + seeded_offset(seed ^ 0xD00D) * 18.0, t) * 0.07)
        * (-t * 8.6).exp();
    ((grain * 0.42 + rasp * 0.22) * drag + ring) * (-t * 5.8).exp() * velocity * 0.34
}

fn tabla_low(t: f32, velocity: f32, seed: u64) -> f32 {
    let bend = 86.0 * (1.0 + 0.7 * (-t * 17.0).exp());
    let tone = sine(bend, t) + sine(bend * 2.01, t) * 0.18;
    tone * (-t * 7.2).exp() * velocity * (0.78 + seeded_offset(seed).abs() * 0.06)
}

fn tabla_tuned(t: f32, velocity: f32, frequency: f32, seed: u64) -> f32 {
    let tone =
        sine(frequency * (1.0 + seeded_offset(seed) * 0.006), t) + sine(frequency * 2.0, t) * 0.22;
    let click = triangle(frequency * 3.1, t) * (-t * 75.0).exp() * 0.2;
    (tone * (-t * 15.0).exp() + click) * velocity * 0.44
}

fn adsr(t: f32, duration: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> f32 {
    if t < 0.0 {
        return 0.0;
    }
    if t < attack {
        return t / attack.max(0.0001);
    }
    if t < attack + decay {
        let x = (t - attack) / decay.max(0.0001);
        return 1.0 + (sustain - 1.0) * x;
    }
    if t < duration {
        return sustain;
    }
    let release_t = t - duration;
    if release_t >= release {
        0.0
    } else {
        sustain * (1.0 - release_t / release.max(0.0001)).powf(1.5)
    }
}

fn sine(frequency: f32, t: f32) -> f32 {
    (TAU * frequency * t).sin()
}

fn triangle(frequency: f32, t: f32) -> f32 {
    (2.0 / core::f32::consts::PI) * (TAU * frequency * t).sin().asin()
}

fn saw(frequency: f32, t: f32) -> f32 {
    let phase = (frequency * t).fract();
    2.0 * phase - 1.0
}

fn square_pw(frequency: f32, t: f32, width: f32) -> f32 {
    if (frequency * t).fract() < width.clamp(0.08, 0.92) {
        1.0
    } else {
        -1.0
    }
}

fn noise(seed: u64, sample_index: u64) -> f32 {
    let mut x = seed ^ sample_index.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    x ^= x >> 33;
    let value = ((x >> 40) as u32) as f32 / 16_777_215.0;
    value * 2.0 - 1.0
}

fn smooth_noise(seed: u64, sample_index: u64, width: u64) -> f32 {
    let width = width.max(1);
    let center = sample_index / width;
    (noise(seed, center) + noise(seed, center + 1) + noise(seed, center + 2)) / 3.0
}

fn seeded_offset(seed: u64) -> f32 {
    let value = ((seed ^ (seed >> 32)) & 0xffff) as f32 / 65535.0;
    value * 2.0 - 1.0
}

use crate::instruments::InstrumentBank;
use crate::music::{midi_to_frequency, INSTRUMENT_SLOTS, LOOP_BEATS, PERCUSSION_STEPS};
use crate::percussion::PercussionVoiceKind;
use crate::sequence::{generate_band, Band};
use crate::synth::{
    instrument_tail_seconds, percussion_tail_seconds, render_instrument, render_percussion,
};
use crate::utils::{clamp_unit, equal_power_pan, seconds_to_samples, soft_limit};

const MAX_RENDER_FRAMES: usize = 4096;
const INSTRUMENT_ECHO_SECONDS: f32 = 0.185;
const PERCUSSION_ECHO_SECONDS: f32 = 0.115;
const OUTPUT_DRIVE: f32 = 0.58;

#[derive(Clone, Debug)]
struct InstrumentRenderEvent {
    bank: InstrumentBank,
    start_sample: usize,
    duration_samples: usize,
    total_samples: usize,
    frequency: f32,
    velocity: f32,
    pan: f32,
    event_seed: u64,
}

#[derive(Clone, Debug)]
struct PercussionRenderEvent {
    kind: PercussionVoiceKind,
    start_sample: usize,
    total_samples: usize,
    velocity: f32,
    pan: f32,
    event_seed: u64,
}

pub struct RenderEngine {
    band: Band,
    sample_rate: f32,
    loop_samples: usize,
    position: usize,
    absolute_position: u64,
    output_buffer: Vec<f32>,
    instrument_events: Vec<InstrumentRenderEvent>,
    percussion_events: Vec<PercussionRenderEvent>,
}

impl RenderEngine {
    pub fn new(band: Band, sample_rate: f32) -> Result<Self, String> {
        if !sample_rate.is_finite() || sample_rate < 8_000.0 {
            return Err("Invalid audio sample rate".to_string());
        }
        let loop_seconds = 60.0 / band.tempo_bpm as f32 * LOOP_BEATS as f32;
        let loop_samples = seconds_to_samples(loop_seconds, sample_rate).max(1);
        let instrument_events = build_instrument_events(&band, sample_rate, loop_samples);
        let percussion_events = build_percussion_events(&band, sample_rate, loop_samples);

        Ok(Self {
            band,
            sample_rate,
            loop_samples,
            position: 0,
            absolute_position: 0,
            output_buffer: vec![0.0; MAX_RENDER_FRAMES * 2],
            instrument_events,
            percussion_events,
        })
    }

    pub fn from_seed(seed: u64, sample_rate: f32) -> Result<Self, String> {
        Self::new(generate_band(seed), sample_rate)
    }

    pub fn reset(&mut self) {
        self.position = 0;
        self.absolute_position = 0;
        self.output_buffer.fill(0.0);
    }

    pub fn render(&mut self, frames: usize) -> Result<(), String> {
        if frames > MAX_RENDER_FRAMES {
            return Err(format!(
                "Requested render block of {frames} frames exceeds maximum {MAX_RENDER_FRAMES}"
            ));
        }

        for frame in 0..frames {
            let (left, right) = self.sample_at(self.position, self.absolute_position);
            let out = frame * 2;
            self.output_buffer[out] = left;
            self.output_buffer[out + 1] = right;
            self.position += 1;
            if self.position >= self.loop_samples {
                self.position = 0;
            }
            self.absolute_position = self.absolute_position.wrapping_add(1);
        }
        Ok(())
    }

    pub fn output(&self) -> &[f32] {
        &self.output_buffer
    }

    pub fn output_ptr(&self) -> *const f32 {
        self.output_buffer.as_ptr()
    }

    pub fn output_len(&self) -> usize {
        self.output_buffer.len()
    }

    pub fn loop_samples(&self) -> usize {
        self.loop_samples
    }

    pub fn band(&self) -> &Band {
        &self.band
    }

    fn sample_at(&self, loop_position: usize, absolute_sample: u64) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for event in &self.instrument_events {
            if let Some(sample_offset) = wrapped_offset(
                loop_position,
                event.start_sample,
                event.total_samples,
                self.loop_samples,
            ) {
                let t = sample_offset as f32 / self.sample_rate;
                let duration = event.duration_samples as f32 / self.sample_rate;
                let dry = render_instrument(
                    event.bank,
                    event.frequency,
                    t,
                    duration,
                    event.velocity,
                    event.event_seed,
                    absolute_sample,
                );
                let echo_offset = seconds_to_samples(INSTRUMENT_ECHO_SECONDS, self.sample_rate);
                let echo = if sample_offset > echo_offset {
                    let echo_t = (sample_offset - echo_offset) as f32 / self.sample_rate;
                    render_instrument(
                        event.bank,
                        event.frequency,
                        echo_t,
                        duration,
                        event.velocity,
                        event.event_seed ^ 0xD31A_900D,
                        absolute_sample.saturating_sub(echo_offset as u64),
                    ) * 0.12
                } else {
                    0.0
                };
                let (l, r) = equal_power_pan(dry + echo, event.pan);
                left += l;
                right += r;
            }
        }

        for event in &self.percussion_events {
            if let Some(sample_offset) = wrapped_offset(
                loop_position,
                event.start_sample,
                event.total_samples,
                self.loop_samples,
            ) {
                let t = sample_offset as f32 / self.sample_rate;
                let dry = render_percussion(
                    event.kind,
                    t,
                    event.velocity,
                    event.event_seed,
                    absolute_sample,
                );
                let echo_offset = seconds_to_samples(PERCUSSION_ECHO_SECONDS, self.sample_rate);
                let echo = if sample_offset > echo_offset {
                    let echo_t = (sample_offset - echo_offset) as f32 / self.sample_rate;
                    render_percussion(
                        event.kind,
                        echo_t,
                        event.velocity,
                        event.event_seed ^ 0xEC40,
                        absolute_sample.saturating_sub(echo_offset as u64),
                    ) * 0.08
                } else {
                    0.0
                };
                let (l, r) = equal_power_pan(dry + echo, event.pan);
                left += l;
                right += r;
            }
        }

        let left = soft_limit(left * OUTPUT_DRIVE);
        let right = soft_limit(right * OUTPUT_DRIVE);
        (clamp_unit(left), clamp_unit(right))
    }
}

pub fn render_loop(seed: u64, sample_rate: f32) -> Result<Vec<f32>, String> {
    let mut engine = RenderEngine::from_seed(seed, sample_rate)?;
    let mut output = vec![0.0; engine.loop_samples() * 2];
    let mut frame_cursor = 0;

    while frame_cursor < engine.loop_samples() {
        let block = (engine.loop_samples() - frame_cursor).min(MAX_RENDER_FRAMES);
        engine.render(block)?;
        let src = &engine.output()[..block * 2];
        let dst = &mut output[frame_cursor * 2..frame_cursor * 2 + block * 2];
        dst.copy_from_slice(src);
        frame_cursor += block;
    }

    Ok(output)
}

fn build_instrument_events(
    band: &Band,
    sample_rate: f32,
    loop_samples: usize,
) -> Vec<InstrumentRenderEvent> {
    let mut events = Vec::new();
    for (part_index, part) in band.instruments.iter().enumerate() {
        let pan = match part_index {
            0 => -0.5,
            1 => 0.0,
            _ => 0.5,
        };
        let tail = seconds_to_samples(
            instrument_tail_seconds(part.bank) + INSTRUMENT_ECHO_SECONDS,
            sample_rate,
        );
        for note in &part.notes {
            let start_sample = slot_to_sample(note.start_slot, INSTRUMENT_SLOTS, loop_samples);
            let end_sample = slot_to_sample(
                note.start_slot + note.duration_slots,
                INSTRUMENT_SLOTS,
                loop_samples,
            );
            let duration_samples = end_sample.saturating_sub(start_sample).max(1);
            events.push(InstrumentRenderEvent {
                bank: part.bank,
                start_sample,
                duration_samples,
                total_samples: duration_samples + tail,
                frequency: midi_to_frequency(note.midi),
                velocity: note.velocity,
                pan,
                event_seed: note.event_seed,
            });
        }
    }
    events
}

fn build_percussion_events(
    band: &Band,
    sample_rate: f32,
    loop_samples: usize,
) -> Vec<PercussionRenderEvent> {
    let mut events = Vec::new();
    for (voice_index, voice) in band.percussion.voices.iter().enumerate() {
        let pan = match voice_index {
            0 => -0.1,
            1 => 0.16,
            2 => -0.36,
            _ => 0.36,
        };
        let tail = seconds_to_samples(
            percussion_tail_seconds(voice.kind) + PERCUSSION_ECHO_SECONDS,
            sample_rate,
        );
        for hit in &voice.hits {
            events.push(PercussionRenderEvent {
                kind: voice.kind,
                start_sample: slot_to_sample(hit.step, PERCUSSION_STEPS, loop_samples),
                total_samples: tail.max(1),
                velocity: hit.velocity,
                pan,
                event_seed: hit.event_seed,
            });
        }
    }
    events
}

fn slot_to_sample(slot: usize, slots: usize, loop_samples: usize) -> usize {
    ((slot as f64 / slots as f64) * loop_samples as f64).round() as usize
}

fn wrapped_offset(
    loop_position: usize,
    start_sample: usize,
    total_samples: usize,
    loop_samples: usize,
) -> Option<usize> {
    let offset = if loop_position >= start_sample {
        loop_position - start_sample
    } else {
        loop_samples - start_sample + loop_position
    };
    (offset < total_samples).then_some(offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_render_length_is_correct() {
        let sample_rate = 48_000.0;
        let seed = 42;
        let engine = RenderEngine::from_seed(seed, sample_rate).unwrap();
        let rendered = render_loop(seed, sample_rate).unwrap();
        assert_eq!(rendered.len(), engine.loop_samples() * 2);
    }

    #[test]
    fn rendered_samples_are_finite_and_bounded() {
        let mut engine = RenderEngine::from_seed(777, 44_100.0).unwrap();
        engine.render(1024).unwrap();
        for sample in &engine.output()[..2048] {
            assert!(sample.is_finite());
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn generated_audio_is_not_silent_under_normal_conditions() {
        let rendered = render_loop(1234, 24_000.0).unwrap();
        let energy: f32 = rendered
            .iter()
            .take(24_000)
            .map(|sample| sample.abs())
            .sum();
        assert!(energy > 1.0);
    }
}

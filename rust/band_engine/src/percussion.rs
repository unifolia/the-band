use serde::Serialize;

use crate::euclid::{euclidean_pattern, rotate_pattern};
use crate::music::{PERCUSSION_STEPS, PERCUSSION_STEPS_PER_BAR};
use crate::rng::Rng64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PercussionBank {
    Drums,
    Woods,
    Cans,
    Tabla,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PercussionVoiceKind {
    Kick,
    Snare,
    Hat,
    Tom,
    Woodblock,
    Clave,
    LogDrum,
    Shaker,
    CanKick,
    CanSnare,
    Scrape,
    AltScrape,
    Bayan,
    Dayan,
    NaTin,
    MutedTap,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PercussionHit {
    pub step: usize,
    pub velocity: f32,
    pub event_seed: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PercussionVoice {
    pub kind: PercussionVoiceKind,
    pub pulses: usize,
    pub rotation: usize,
    pub probability: f32,
    pub pattern: Vec<bool>,
    pub hits: Vec<PercussionHit>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PercussionPart {
    pub bank: PercussionBank,
    pub voices: Vec<PercussionVoice>,
}

impl PercussionBank {
    pub const ALL: [PercussionBank; 4] = [
        PercussionBank::Drums,
        PercussionBank::Woods,
        PercussionBank::Cans,
        PercussionBank::Tabla,
    ];

    pub fn choose(rng: &mut Rng64) -> Self {
        Self::ALL[rng.pick_index(Self::ALL.len())]
    }

    fn voices(self) -> &'static [PercussionVoiceKind] {
        match self {
            PercussionBank::Drums => &[
                PercussionVoiceKind::Kick,
                PercussionVoiceKind::Snare,
                PercussionVoiceKind::Hat,
                PercussionVoiceKind::Tom,
            ],
            PercussionBank::Woods => &[
                PercussionVoiceKind::Woodblock,
                PercussionVoiceKind::Clave,
                PercussionVoiceKind::LogDrum,
                PercussionVoiceKind::Shaker,
            ],
            PercussionBank::Cans => &[
                PercussionVoiceKind::CanKick,
                PercussionVoiceKind::CanSnare,
                PercussionVoiceKind::Scrape,
                PercussionVoiceKind::AltScrape,
            ],
            PercussionBank::Tabla => &[
                PercussionVoiceKind::Bayan,
                PercussionVoiceKind::Dayan,
                PercussionVoiceKind::NaTin,
                PercussionVoiceKind::MutedTap,
            ],
        }
    }
}

pub fn generate_percussion(bank: PercussionBank, rng: &mut Rng64) -> PercussionPart {
    let voices = bank
        .voices()
        .iter()
        .enumerate()
        .map(|(voice_index, kind)| generate_voice(*kind, voice_index, rng))
        .collect();
    PercussionPart { bank, voices }
}

fn generate_voice(
    kind: PercussionVoiceKind,
    voice_index: usize,
    rng: &mut Rng64,
) -> PercussionVoice {
    let (pulse_min, pulse_max, probability_min, probability_max) = voice_bounds(kind);
    let pulses = rng.range_usize(pulse_min, pulse_max);
    let rotation = rng.range_usize(0, PERCUSSION_STEPS_PER_BAR - 1);
    let probability = rng.range_f32(probability_min, probability_max);
    let base = rotate_pattern(
        &euclidean_pattern(PERCUSSION_STEPS_PER_BAR, pulses),
        rotation,
    );
    let mut pattern = vec![false; PERCUSSION_STEPS];
    let mut hits = Vec::new();

    for bar in 0..4 {
        let bar_offset = bar * PERCUSSION_STEPS_PER_BAR;
        for step in 0..PERCUSSION_STEPS_PER_BAR {
            if !base[step] {
                continue;
            }
            let strong_step = step % 4 == 0;
            let keep_probability = if strong_step {
                (probability + 0.12).min(1.0)
            } else {
                probability
            };
            if rng.chance(keep_probability) {
                let absolute_step = bar_offset + step;
                let accent = if strong_step {
                    1.0
                } else {
                    rng.range_f32(0.68, 0.92)
                };
                let bar_lift = if bar == 0 {
                    1.0
                } else {
                    rng.range_f32(0.88, 1.03)
                };
                pattern[absolute_step] = true;
                hits.push(PercussionHit {
                    step: absolute_step,
                    velocity: (accent * bar_lift).min(1.0),
                    event_seed: rng.next_u64() ^ ((voice_index as u64) << 32),
                });
            }
        }
    }

    if hits.is_empty() {
        let step = rotation % PERCUSSION_STEPS_PER_BAR;
        pattern[step] = true;
        hits.push(PercussionHit {
            step,
            velocity: 0.9,
            event_seed: rng.next_u64() ^ ((voice_index as u64) << 32),
        });
    }

    PercussionVoice {
        kind,
        pulses,
        rotation,
        probability,
        pattern,
        hits,
    }
}

fn voice_bounds(kind: PercussionVoiceKind) -> (usize, usize, f32, f32) {
    match kind {
        PercussionVoiceKind::Kick | PercussionVoiceKind::CanKick | PercussionVoiceKind::Bayan => {
            (3, 5, 0.82, 1.0)
        }
        PercussionVoiceKind::Snare | PercussionVoiceKind::CanSnare | PercussionVoiceKind::Dayan => {
            (2, 4, 0.76, 0.95)
        }
        PercussionVoiceKind::Hat | PercussionVoiceKind::Shaker | PercussionVoiceKind::NaTin => {
            (7, 11, 0.72, 0.93)
        }
        PercussionVoiceKind::Tom | PercussionVoiceKind::LogDrum | PercussionVoiceKind::MutedTap => {
            (2, 5, 0.68, 0.9)
        }
        PercussionVoiceKind::Woodblock | PercussionVoiceKind::Clave => (4, 7, 0.7, 0.94),
        PercussionVoiceKind::Scrape | PercussionVoiceKind::AltScrape => (4, 8, 0.62, 0.86),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percussion_banks_have_expected_voices_and_are_not_silent() {
        for bank in [
            PercussionBank::Drums,
            PercussionBank::Woods,
            PercussionBank::Cans,
            PercussionBank::Tabla,
        ] {
            let mut rng = Rng64::new(bank as u64 + 77);
            let part = generate_percussion(bank, &mut rng);
            assert_eq!(part.voices.len(), bank.voices().len());
            assert!(part.voices.iter().any(|voice| !voice.hits.is_empty()));
            assert!(part
                .voices
                .iter()
                .all(|voice| voice.pattern.len() == PERCUSSION_STEPS));
        }
    }

    #[test]
    fn cans_bank_uses_current_four_voice_set() {
        assert_eq!(
            PercussionBank::Cans.voices(),
            [
                PercussionVoiceKind::CanKick,
                PercussionVoiceKind::CanSnare,
                PercussionVoiceKind::Scrape,
                PercussionVoiceKind::AltScrape,
            ]
        );
    }

    #[test]
    fn percussion_patterns_are_controlled() {
        let mut rng = Rng64::new(99);
        let part = generate_percussion(PercussionBank::Drums, &mut rng);
        for voice in part.voices {
            assert!(voice.pulses > 0);
            assert!(voice.pulses < PERCUSSION_STEPS_PER_BAR);
            assert!(voice.probability >= 0.6);
            assert!(voice.probability <= 1.0);
        }
    }
}

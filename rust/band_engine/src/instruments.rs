use serde::Serialize;

use crate::rng::Rng64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum InstrumentBank {
    Harp,
    Bass,
    Pads,
    Synth,
    Clarinet,
}

#[derive(Clone, Copy, Debug)]
pub struct InstrumentProfile {
    pub low_midi: u8,
    pub high_midi: u8,
    pub density: f32,
    pub min_sustain_slots: usize,
    pub max_sustain_slots: usize,
    pub max_leap_semitones: u8,
    pub release_seconds: f32,
}

impl InstrumentBank {
    pub const ALL: [InstrumentBank; 5] = [
        InstrumentBank::Harp,
        InstrumentBank::Bass,
        InstrumentBank::Pads,
        InstrumentBank::Synth,
        InstrumentBank::Clarinet,
    ];

    pub fn choose(rng: &mut Rng64) -> Self {
        Self::ALL[rng.pick_index(Self::ALL.len())]
    }

    pub fn profile(self) -> InstrumentProfile {
        match self {
            InstrumentBank::Harp => InstrumentProfile {
                low_midi: 48,
                high_midi: 84,
                density: 0.62,
                min_sustain_slots: 1,
                max_sustain_slots: 2,
                max_leap_semitones: 7,
                release_seconds: 0.32,
            },
            InstrumentBank::Bass => InstrumentProfile {
                low_midi: 24,
                high_midi: 48,
                density: 0.28,
                min_sustain_slots: 1,
                max_sustain_slots: 3,
                max_leap_semitones: 7,
                release_seconds: 0.14,
            },
            InstrumentBank::Pads => InstrumentProfile {
                low_midi: 36,
                high_midi: 72,
                density: 0.34,
                min_sustain_slots: 3,
                max_sustain_slots: 8,
                max_leap_semitones: 5,
                release_seconds: 0.72,
            },
            InstrumentBank::Synth => InstrumentProfile {
                low_midi: 48,
                high_midi: 84,
                density: 0.48,
                min_sustain_slots: 1,
                max_sustain_slots: 3,
                max_leap_semitones: 7,
                release_seconds: 0.2,
            },
            InstrumentBank::Clarinet => InstrumentProfile {
                low_midi: 50,
                high_midi: 81,
                density: 0.42,
                min_sustain_slots: 2,
                max_sustain_slots: 4,
                max_leap_semitones: 7,
                release_seconds: 0.18,
            },
        }
    }
}

pub fn select_instrument_banks(rng: &mut Rng64) -> [InstrumentBank; 3] {
    [
        InstrumentBank::choose(rng),
        InstrumentBank::choose(rng),
        InstrumentBank::choose(rng),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng64;

    #[test]
    fn independent_instrument_selection_can_duplicate() {
        let mut found_duplicate = false;
        for seed in 1..2048 {
            let mut rng = Rng64::new(seed);
            let banks = select_instrument_banks(&mut rng);
            if banks[0] == banks[1] || banks[0] == banks[2] || banks[1] == banks[2] {
                found_duplicate = true;
                break;
            }
        }
        assert!(found_duplicate);
    }

    #[test]
    fn bass_density_is_lower_than_typical_melodic_banks() {
        let bass = InstrumentBank::Bass.profile().density;
        assert!(bass < InstrumentBank::Harp.profile().density);
        assert!(bass < InstrumentBank::Synth.profile().density);
        assert!(bass < InstrumentBank::Clarinet.profile().density);
    }

    #[test]
    fn pads_sustain_longer_than_plucked_harp() {
        let pads = InstrumentBank::Pads.profile();
        let harp = InstrumentBank::Harp.profile();
        assert!(pads.min_sustain_slots > harp.min_sustain_slots);
        assert!(pads.max_sustain_slots > harp.max_sustain_slots);
    }
}

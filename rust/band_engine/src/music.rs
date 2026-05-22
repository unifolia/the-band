use serde::Serialize;

use crate::rng::Rng64;

pub const INSTRUMENT_SLOTS: usize = 32;
pub const PERCUSSION_STEPS_PER_BAR: usize = 16;
pub const PERCUSSION_STEPS: usize = 64;
pub const LOOP_BARS: usize = 4;
pub const BEATS_PER_BAR: usize = 4;
pub const LOOP_BEATS: usize = LOOP_BARS * BEATS_PER_BAR;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Mode {
    Ionian,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
}

impl Mode {
    pub const ALL: [Mode; 6] = [
        Mode::Ionian,
        Mode::Dorian,
        Mode::Phrygian,
        Mode::Lydian,
        Mode::Mixolydian,
        Mode::Aeolian,
    ];

    pub fn intervals(self) -> [u8; 7] {
        match self {
            Mode::Ionian => [0, 2, 4, 5, 7, 9, 11],
            Mode::Dorian => [0, 2, 3, 5, 7, 9, 10],
            Mode::Phrygian => [0, 1, 3, 5, 7, 8, 10],
            Mode::Lydian => [0, 2, 4, 6, 7, 9, 11],
            Mode::Mixolydian => [0, 2, 4, 5, 7, 9, 10],
            Mode::Aeolian => [0, 2, 3, 5, 7, 8, 10],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootNote {
    pub pitch_class: u8,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarmonicBar {
    pub bar: usize,
    pub root_degree: u8,
    pub chord_degrees: [u8; 3],
}

const ROOT_NAMES: [&[&str]; 12] = [
    &["C"],
    &["C#", "Db"],
    &["D"],
    &["D#", "Eb"],
    &["E", "Fb"],
    &["F", "E#"],
    &["F#", "Gb"],
    &["G"],
    &["G#", "Ab"],
    &["A"],
    &["A#", "Bb"],
    &["B", "Cb"],
];

pub fn choose_root(rng: &mut Rng64) -> RootNote {
    let pitch_class = rng.range_u32(0, 11) as u8;
    let spellings = ROOT_NAMES[pitch_class as usize];
    let name = spellings[rng.pick_index(spellings.len())].to_string();
    RootNote { pitch_class, name }
}

pub fn choose_mode(rng: &mut Rng64) -> Mode {
    Mode::ALL[rng.pick_index(Mode::ALL.len())]
}

pub fn generate_harmony(rng: &mut Rng64) -> Vec<HarmonicBar> {
    let stable_degrees = [0u8, 4, 2, 5, 3];
    let mut bars = Vec::with_capacity(LOOP_BARS);

    for bar in 0..LOOP_BARS {
        let root_degree = if bar == 0 {
            0
        } else if bar == LOOP_BARS - 1 && rng.chance(0.45) {
            4
        } else {
            stable_degrees[rng.pick_index(stable_degrees.len())]
        };
        bars.push(HarmonicBar {
            bar,
            root_degree,
            chord_degrees: [
                root_degree % 7,
                (root_degree + 2) % 7,
                (root_degree + 4) % 7,
            ],
        });
    }

    bars
}

pub fn midi_to_frequency(midi: u8) -> f32 {
    440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
}

pub fn pitch_class_for_midi(midi: u8) -> u8 {
    midi % 12
}

pub fn mode_degree_for_midi(midi: u8, root_pitch_class: u8, mode: Mode) -> Option<u8> {
    let relative = (12 + pitch_class_for_midi(midi) as i16 - root_pitch_class as i16) % 12;
    mode.intervals()
        .iter()
        .position(|interval| *interval as i16 == relative)
        .map(|degree| degree as u8)
}

pub fn midi_is_in_mode(midi: u8, root_pitch_class: u8, mode: Mode) -> bool {
    mode_degree_for_midi(midi, root_pitch_class, mode).is_some()
}

pub fn mode_midis_in_range(low: u8, high: u8, root_pitch_class: u8, mode: Mode) -> Vec<u8> {
    (low..=high)
        .filter(|midi| midi_is_in_mode(*midi, root_pitch_class, mode))
        .collect()
}

pub fn midi_is_chord_tone(
    midi: u8,
    root_pitch_class: u8,
    mode: Mode,
    harmonic_bar: &HarmonicBar,
) -> bool {
    mode_degree_for_midi(midi, root_pitch_class, mode)
        .map(|degree| harmonic_bar.chord_degrees.contains(&degree))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_intervals_are_correct() {
        assert_eq!(Mode::Ionian.intervals(), [0, 2, 4, 5, 7, 9, 11]);
        assert_eq!(Mode::Dorian.intervals(), [0, 2, 3, 5, 7, 9, 10]);
        assert_eq!(Mode::Phrygian.intervals(), [0, 1, 3, 5, 7, 8, 10]);
        assert_eq!(Mode::Lydian.intervals(), [0, 2, 4, 6, 7, 9, 11]);
        assert_eq!(Mode::Mixolydian.intervals(), [0, 2, 4, 5, 7, 9, 10]);
        assert_eq!(Mode::Aeolian.intervals(), [0, 2, 3, 5, 7, 8, 10]);
    }

    #[test]
    fn locrian_is_not_an_allowed_mode() {
        assert_eq!(Mode::ALL.len(), 6);
        let names = Mode::ALL.map(|mode| format!("{mode:?}"));
        assert!(!names.iter().any(|name| name == "Locrian"));
    }

    #[test]
    fn a4_frequency_is_440_hz() {
        assert!((midi_to_frequency(69) - 440.0).abs() < 0.001);
    }
}

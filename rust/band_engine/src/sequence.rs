use serde::Serialize;

use crate::instruments::{select_instrument_banks, InstrumentBank};
use crate::music::{
    choose_mode, choose_root, generate_harmony, midi_is_chord_tone, mode_degree_for_midi,
    mode_midis_in_range, HarmonicBar, Mode, RootNote, INSTRUMENT_SLOTS, LOOP_BEATS,
};
use crate::percussion::{generate_percussion, PercussionBank, PercussionPart};
use crate::rng::{mix64, Rng64};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MemberRole {
    Percussionist,
    Instrumentalist,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BandMember {
    pub index: usize,
    pub role: MemberRole,
    pub percussion_bank: Option<PercussionBank>,
    pub instrument_bank: Option<InstrumentBank>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SlotKind {
    Rest,
    Note,
    Tie,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotCell {
    pub kind: SlotKind,
    pub midi: Option<u8>,
    pub duration_slots: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteEvent {
    pub start_slot: usize,
    pub duration_slots: usize,
    pub midi: u8,
    pub velocity: f32,
    pub event_seed: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentPart {
    pub member_index: usize,
    pub bank: InstrumentBank,
    pub slots: Vec<SlotCell>,
    pub notes: Vec<NoteEvent>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Band {
    pub seed: u64,
    pub tempo_bpm: u32,
    pub root: RootNote,
    pub mode: Mode,
    pub loop_beats: usize,
    pub members: Vec<BandMember>,
    pub harmonic_framework: Vec<HarmonicBar>,
    pub percussion: PercussionPart,
    pub instruments: Vec<InstrumentPart>,
}

pub fn generate_band(seed: u64) -> Band {
    let mut rng = Rng64::from_seed_and_salt(seed, 0xBADC_0FFE_E0DD_F00D);
    let tempo_bpm = rng.range_u32(80, 150);
    let root = choose_root(&mut rng);
    let mode = choose_mode(&mut rng);
    let harmonic_framework = generate_harmony(&mut rng);

    let percussion_bank = PercussionBank::choose(&mut rng);
    let mut percussion_rng = Rng64::from_seed_and_salt(seed, 0xC0DA_5EED_0000_0001);
    let percussion = generate_percussion(percussion_bank, &mut percussion_rng);

    let mut instrument_rng = Rng64::from_seed_and_salt(seed, 0xC0DA_5EED_0000_0002);
    let instrument_banks = select_instrument_banks(&mut instrument_rng);
    let instruments = instrument_banks
        .iter()
        .enumerate()
        .map(|(index, bank)| {
            let mut phrase_rng = Rng64::from_seed_and_salt(
                seed,
                0x1A57_0000_0000_0000 ^ ((index as u64 + 1) * 0x101),
            );
            generate_phrase(
                index + 1,
                *bank,
                root.pitch_class,
                mode,
                &harmonic_framework,
                &mut phrase_rng,
            )
        })
        .collect::<Vec<_>>();

    let mut members = Vec::with_capacity(4);
    members.push(BandMember {
        index: 0,
        role: MemberRole::Percussionist,
        percussion_bank: Some(percussion_bank),
        instrument_bank: None,
    });
    for (member_offset, bank) in instrument_banks.iter().enumerate() {
        members.push(BandMember {
            index: member_offset + 1,
            role: MemberRole::Instrumentalist,
            percussion_bank: None,
            instrument_bank: Some(*bank),
        });
    }

    Band {
        seed,
        tempo_bpm,
        root,
        mode,
        loop_beats: LOOP_BEATS,
        members,
        harmonic_framework,
        percussion,
        instruments,
    }
}

fn generate_phrase(
    member_index: usize,
    bank: InstrumentBank,
    root_pitch_class: u8,
    mode: Mode,
    harmony: &[HarmonicBar],
    rng: &mut Rng64,
) -> InstrumentPart {
    let profile = bank.profile();
    let mut slots = vec![
        SlotCell {
            kind: SlotKind::Rest,
            midi: None,
            duration_slots: None,
        };
        INSTRUMENT_SLOTS
    ];
    let mut notes = Vec::new();
    let mut previous_midi = None;
    let mut slot = 0;

    while slot < INSTRUMENT_SLOTS {
        let bar = (slot / 8).min(harmony.len() - 1);
        let slot_in_bar = slot % 8;
        let mut density = profile.density;
        density *= match bank {
            InstrumentBank::Pads if slot_in_bar % 2 == 1 => 0.2,
            InstrumentBank::Bass if slot_in_bar % 2 == 1 => 0.65,
            InstrumentBank::Clarinet if slot_in_bar == 7 => 0.25,
            _ => 1.0,
        };

        if rng.chance(density) {
            let remaining = INSTRUMENT_SLOTS - slot;
            let duration = choose_duration(bank, remaining, rng);
            let midi = choose_pitch(
                bank,
                previous_midi,
                root_pitch_class,
                mode,
                &harmony[bar],
                slot,
                rng,
            );
            let velocity = match bank {
                InstrumentBank::Bass => rng.range_f32(0.62, 0.86),
                InstrumentBank::Pads => rng.range_f32(0.42, 0.66),
                InstrumentBank::Harp => rng.range_f32(0.44, 0.76),
                InstrumentBank::Synth => rng.range_f32(0.46, 0.74),
                InstrumentBank::Clarinet => rng.range_f32(0.5, 0.78),
            };
            slots[slot] = SlotCell {
                kind: SlotKind::Note,
                midi: Some(midi),
                duration_slots: Some(duration),
            };
            for tie_slot in slot + 1..(slot + duration).min(INSTRUMENT_SLOTS) {
                slots[tie_slot] = SlotCell {
                    kind: SlotKind::Tie,
                    midi: Some(midi),
                    duration_slots: None,
                };
            }
            notes.push(NoteEvent {
                start_slot: slot,
                duration_slots: duration,
                midi,
                velocity,
                event_seed: mix64(rng.next_u64() ^ ((member_index as u64) << 40)),
            });
            previous_midi = Some(midi);
            slot += duration;
            if bank == InstrumentBank::Clarinet && slot < INSTRUMENT_SLOTS && rng.chance(0.45) {
                slot += 1;
            }
        } else {
            slot += 1;
        }
    }

    if notes.is_empty() {
        let midi = choose_pitch(bank, None, root_pitch_class, mode, &harmony[0], 0, rng);
        slots[0] = SlotCell {
            kind: SlotKind::Note,
            midi: Some(midi),
            duration_slots: Some(profile.min_sustain_slots),
        };
        for tie_slot in 1..profile.min_sustain_slots.min(INSTRUMENT_SLOTS) {
            slots[tie_slot] = SlotCell {
                kind: SlotKind::Tie,
                midi: Some(midi),
                duration_slots: None,
            };
        }
        notes.push(NoteEvent {
            start_slot: 0,
            duration_slots: profile.min_sustain_slots,
            midi,
            velocity: 0.62,
            event_seed: mix64(rng.next_u64() ^ ((member_index as u64) << 40)),
        });
    }

    InstrumentPart {
        member_index,
        bank,
        slots,
        notes,
    }
}

fn choose_duration(bank: InstrumentBank, remaining: usize, rng: &mut Rng64) -> usize {
    let profile = bank.profile();
    let max = profile.max_sustain_slots.min(remaining).max(1);
    let min = profile.min_sustain_slots.min(max);
    let mut duration = rng.range_usize(min, max);

    if bank == InstrumentBank::Harp && rng.chance(0.72) {
        duration = 1;
    }
    if bank == InstrumentBank::Pads && rng.chance(0.35) {
        duration = max;
    }
    duration.max(1)
}

fn choose_pitch(
    bank: InstrumentBank,
    previous_midi: Option<u8>,
    root_pitch_class: u8,
    mode: Mode,
    harmonic_bar: &HarmonicBar,
    slot: usize,
    rng: &mut Rng64,
) -> u8 {
    let profile = bank.profile();
    let all_candidates =
        mode_midis_in_range(profile.low_midi, profile.high_midi, root_pitch_class, mode);
    let bounded_candidates = if let Some(previous) = previous_midi {
        let bounded = all_candidates
            .iter()
            .copied()
            .filter(|midi| previous.abs_diff(*midi) <= profile.max_leap_semitones)
            .collect::<Vec<_>>();
        if bounded.is_empty() {
            all_candidates.clone()
        } else {
            bounded
        }
    } else {
        all_candidates.clone()
    };

    let strong_slot = slot % 4 == 0 || slot % 8 == 0;
    let center = match bank {
        InstrumentBank::Bass => 36.0,
        InstrumentBank::Pads => 54.0,
        InstrumentBank::Harp | InstrumentBank::Synth => 66.0,
        InstrumentBank::Clarinet => 62.0,
    };

    let mut weighted = Vec::with_capacity(bounded_candidates.len());
    let mut total = 0.0;

    for midi in bounded_candidates {
        let degree = mode_degree_for_midi(midi, root_pitch_class, mode).unwrap_or(0);
        let chord_tone = midi_is_chord_tone(midi, root_pitch_class, mode, harmonic_bar);
        let mut weight = 1.0;

        if chord_tone {
            weight *= if strong_slot { 3.6 } else { 2.0 };
        } else {
            weight *= if strong_slot { 0.32 } else { 1.15 };
        }

        if bank == InstrumentBank::Bass {
            let root_degree = harmonic_bar.root_degree % 7;
            let fifth_degree = (harmonic_bar.root_degree + 4) % 7;
            if degree == root_degree {
                weight *= 3.4;
            } else if degree == fifth_degree {
                weight *= 2.0;
            } else {
                weight *= 0.45;
            }
        }

        if let Some(previous) = previous_midi {
            let distance = previous.abs_diff(midi);
            weight *= match distance {
                0 => 1.45,
                1..=2 => 2.7,
                3..=4 => 1.85,
                5..=7 => 0.8,
                _ => 0.05,
            };
        } else {
            let distance_from_center = (midi as f32 - center).abs();
            weight *= (1.0 / (1.0 + distance_from_center / 12.0)).max(0.2);
        }

        total += weight;
        weighted.push((midi, weight));
    }

    let mut cursor = rng.range_f32(0.0, total.max(0.001));
    for (midi, weight) in weighted {
        if cursor <= weight {
            return midi;
        }
        cursor -= weight;
    }

    all_candidates[all_candidates.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::midi_is_in_mode;

    #[test]
    fn generated_band_has_required_roles_and_ranges() {
        for seed in 1..96 {
            let band = generate_band(seed);
            assert!((80..=150).contains(&band.tempo_bpm));
            assert_eq!(band.members.len(), 4);
            assert_eq!(band.members[0].role, MemberRole::Percussionist);
            assert_eq!(
                band.members
                    .iter()
                    .filter(|member| member.role == MemberRole::Instrumentalist)
                    .count(),
                3
            );
            assert_eq!(band.instruments.len(), 3);
            for part in &band.instruments {
                assert_eq!(part.slots.len(), INSTRUMENT_SLOTS);
                let profile = part.bank.profile();
                for note in &part.notes {
                    assert!(note.start_slot < INSTRUMENT_SLOTS);
                    assert!(note.duration_slots >= 1);
                    assert!(note.start_slot + note.duration_slots <= INSTRUMENT_SLOTS);
                    assert!((profile.low_midi..=profile.high_midi).contains(&note.midi));
                    assert!(midi_is_in_mode(note.midi, band.root.pitch_class, band.mode));
                }
            }
        }
    }

    #[test]
    fn melodic_leaps_are_bounded() {
        for seed in 100..180 {
            let band = generate_band(seed);
            for part in &band.instruments {
                let max_leap = part.bank.profile().max_leap_semitones;
                for pair in part.notes.windows(2) {
                    assert!(
                        pair[0].midi.abs_diff(pair[1].midi) <= max_leap,
                        "{:?} leap {} -> {} exceeded {}",
                        part.bank,
                        pair[0].midi,
                        pair[1].midi,
                        max_leap
                    );
                }
            }
        }
    }

    #[test]
    fn generated_band_is_not_musically_silent() {
        for seed in 1..32 {
            let band = generate_band(seed);
            assert!(band
                .percussion
                .voices
                .iter()
                .any(|voice| !voice.hits.is_empty()));
            assert!(band.instruments.iter().all(|part| !part.notes.is_empty()));
        }
    }
}

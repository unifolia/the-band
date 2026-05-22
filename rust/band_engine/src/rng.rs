const SPLITMIX_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Debug)]
pub struct Rng64 {
    state: u64,
}

impl Rng64 {
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0xA5A5_1F2E_3D4C_5B6A } else { seed };
        Self { state }
    }

    pub fn from_seed_and_salt(seed: u64, salt: u64) -> Self {
        Self::new(mix64(seed ^ salt.wrapping_mul(SPLITMIX_INCREMENT)))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_INCREMENT);
        mix64(self.state)
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        let value = (self.next_u32() >> 8) as f32;
        value / 16_777_216.0
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability.clamp(0.0, 1.0)
    }

    pub fn range_u32(&mut self, min: u32, max_inclusive: u32) -> u32 {
        debug_assert!(min <= max_inclusive);
        let span = max_inclusive - min + 1;
        min + (self.next_u32() % span)
    }

    pub fn range_usize(&mut self, min: usize, max_inclusive: usize) -> usize {
        debug_assert!(min <= max_inclusive);
        let span = max_inclusive - min + 1;
        min + (self.next_u64() as usize % span)
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    pub fn signed_unit(&mut self) -> f32 {
        self.range_f32(-1.0, 1.0)
    }

    pub fn pick_index(&mut self, len: usize) -> usize {
        debug_assert!(len > 0);
        self.range_usize(0, len - 1)
    }
}

pub fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_rng_is_deterministic() {
        let mut a = Rng64::new(12345);
        let mut b = Rng64::new(12345);
        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng64::new(12345);
        let mut b = Rng64::new(54321);
        assert_ne!(a.next_u64(), b.next_u64());
    }
}

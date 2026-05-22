pub fn euclidean_pattern(steps: usize, pulses: usize) -> Vec<bool> {
    if steps == 0 {
        return Vec::new();
    }
    if pulses == 0 {
        return vec![false; steps];
    }
    if pulses >= steps {
        return vec![true; steps];
    }

    let mut counts = Vec::new();
    let mut remainders = vec![pulses];
    let mut divisor = steps - pulses;
    let mut level = 0;

    loop {
        counts.push(divisor / remainders[level]);
        remainders.push(divisor % remainders[level]);
        divisor = remainders[level];
        level += 1;
        if remainders[level] <= 1 {
            break;
        }
    }
    counts.push(divisor);

    let mut pattern = Vec::with_capacity(steps);
    build_pattern(level as isize, &counts, &remainders, &mut pattern);
    pattern.truncate(steps);

    if let Some(first_hit) = pattern.iter().position(|hit| *hit) {
        pattern.rotate_left(first_hit);
    }
    pattern
}

fn build_pattern(level: isize, counts: &[usize], remainders: &[usize], pattern: &mut Vec<bool>) {
    if level == -1 {
        pattern.push(false);
        return;
    }
    if level == -2 {
        pattern.push(true);
        return;
    }

    let index = level as usize;
    for _ in 0..counts[index] {
        build_pattern(level - 1, counts, remainders, pattern);
    }
    if remainders[index] != 0 {
        build_pattern(level - 2, counts, remainders, pattern);
    }
}

pub fn rotate_pattern(pattern: &[bool], rotation: usize) -> Vec<bool> {
    if pattern.is_empty() {
        return Vec::new();
    }
    let mut rotated = pattern.to_vec();
    rotated.rotate_right(rotation % pattern.len());
    rotated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclidean_three_over_eight_is_balanced() {
        assert_eq!(
            euclidean_pattern(8, 3),
            vec![true, false, false, true, false, false, true, false]
        );
    }

    #[test]
    fn euclidean_four_over_sixteen_is_even() {
        let pattern = euclidean_pattern(16, 4);
        let hits: Vec<usize> = pattern
            .iter()
            .enumerate()
            .filter_map(|(index, hit)| hit.then_some(index))
            .collect();
        assert_eq!(hits, vec![0, 4, 8, 12]);
    }

    #[test]
    fn euclidean_handles_empty_and_full_cases() {
        assert_eq!(euclidean_pattern(5, 0), vec![false; 5]);
        assert_eq!(euclidean_pattern(5, 9), vec![true; 5]);
    }

    #[test]
    fn rotation_preserves_hit_count() {
        let pattern = euclidean_pattern(16, 5);
        let rotated = rotate_pattern(&pattern, 3);
        assert_eq!(
            pattern.iter().filter(|hit| **hit).count(),
            rotated.iter().filter(|hit| **hit).count()
        );
    }
}

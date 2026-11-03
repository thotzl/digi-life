//! Payload reader: every trait value is one letter of a gene's payload, picked by a locus shift.

use super::alphabet::char_to_value;

pub fn get_payload_linear_value_offset(payload: &str, shift: usize) -> f32 {
    if payload.is_empty() {
        return 0.5;
    }
    let chars: Vec<char> = payload.chars().collect();
    // Deterministically select exactly one character locus based on the shift offset
    let idx = shift % chars.len();
    let c = chars[idx];

    // Add shift to character value, wrapping modulo 26, to get a beautifully uniform single-character value!
    let val = (char_to_value(c) + shift) % 26;
    val as f32 / 25.0
}

/// One gene payload; an empty payload (missing gene) reads h = 0.5 at every locus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Payload<'a>(pub &'a str);

impl<'a> Payload<'a> {
    /// Value h in [0, 1] at `locus` (`get_payload_linear_value_offset`).
    pub fn h(self, locus: usize) -> f32 {
        get_payload_linear_value_offset(self.0, locus)
    }

    /// `min + h · span`: the linear trait range starting at `min` (written in this order, so values stay bit-identical).
    pub fn range(self, locus: usize, min: f32, span: f32) -> f32 {
        min + self.h(locus) * span
    }

    /// Letter index 0..=25 at `locus` (discrete choices).
    pub fn index(self, locus: usize) -> usize {
        (self.h(locus) * 25.0).round() as usize
    }

    pub fn as_str(self) -> &'a str {
        self.0
    }

    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_allele_mapping() {
        // 1. Verify middle fallback on empty payload
        assert_eq!(get_payload_linear_value_offset("", 0), 0.5);
        assert_eq!(get_payload_linear_value_offset("", 10), 0.5);

        // 2. Verify bounds of different alphabet values
        let val_a = get_payload_linear_value_offset("A", 0);
        let val_z = get_payload_linear_value_offset("Z", 0);
        assert_eq!(val_a, 0.0);
        assert_eq!(val_z, 1.0);

        // 3. Verify perfect mutational continuity: transition from 'O' to 'P'
        let val_o = get_payload_linear_value_offset("O", 0);
        let val_p = get_payload_linear_value_offset("P", 0);
        // 'O' is index 14, 'P' is index 15. The difference should be exactly 1 / 25 = 0.04 (4%)!
        assert!((val_p - val_o - 0.04).abs() < 1e-6);
    }

    #[test]
    fn test_payload_reader_matches_the_free_function() {
        let p = Payload("QWERTYUIOPASDFG");
        for locus in 0..40 {
            assert_eq!(p.h(locus), get_payload_linear_value_offset(p.0, locus));
            assert_eq!(
                p.range(locus, 0.05, 0.85).to_bits(),
                (0.05 + get_payload_linear_value_offset(p.0, locus) * 0.85).to_bits()
            );
        }
    }
}

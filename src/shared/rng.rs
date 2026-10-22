use rand::SeedableRng;

/// Seedable simulation RNG; identical seeds give identical sequences on every platform.
pub type SimRng = rand_chacha::ChaCha8Rng;

/// Deterministic 64-bit seed from a text seed (FNV-1a 64).
pub fn seed_from_str(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn seeded(seed: u64) -> SimRng {
    SimRng::seed_from_u64(seed)
}

/// Independent sub-stream of `seed`, addressed by indices (e.g. [generation, sandbox, trial]).
pub fn derive(seed: u64, stream: &[u64]) -> SimRng {
    let mut state = splitmix64(seed);
    for &index in stream {
        state = splitmix64(state ^ splitmix64(index));
    }
    seeded(state)
}

fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn draw(rng: &mut SimRng) -> Vec<u32> {
        (0..16).map(|_| rng.gen_range(0..1_000_000)).collect()
    }

    #[test]
    fn test_same_seed_same_sequence() {
        let seed = seed_from_str("default_run");
        assert_eq!(seed, seed_from_str("default_run"));
        assert_ne!(seed, seed_from_str("other_run"));
        assert_eq!(draw(&mut seeded(seed)), draw(&mut seeded(seed)));
        assert_eq!(draw(&mut derive(seed, &[3, 7])), draw(&mut derive(seed, &[3, 7])));
    }

    #[test]
    fn test_different_stream_different_sequence() {
        let seed = 42;
        let base = draw(&mut derive(seed, &[1, 2]));
        assert_ne!(base, draw(&mut derive(seed, &[2, 1])));
        assert_ne!(base, draw(&mut derive(seed, &[1, 3])));
        assert_ne!(base, draw(&mut derive(seed, &[1])));
        assert_ne!(base, draw(&mut derive(43, &[1, 2])));
        assert_ne!(draw(&mut seeded(seed)), draw(&mut seeded(seed + 1)));
    }
}

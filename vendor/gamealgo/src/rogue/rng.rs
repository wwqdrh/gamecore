// rng: 可控随机数生成器
//
// 基于 Xoshiro256++ 算法，支持种子复现、权重随机、分布控制
// 提供 fork() 方法派生子 RNG，确保子系统随机互不干扰

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRng {
    s: [u64; 4],
}

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        let mut state = seed;
        let mut s = [0u64; 4];
        for i in 0..4 {
            state = splitmix64(state);
            s[i] = state;
        }
        GameRng { s }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = rotl(self.s[0].wrapping_add(self.s[3]), 23).wrapping_add(self.s[0]);
        let t = self.s[1].wrapping_shl(17);
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = rotl(self.s[3], 45);
        result
    }

    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    pub fn next_range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min) as u64;
        (self.next_u64() % range) as i32 + min
    }

    pub fn next_range_u64(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        let range = max - min;
        self.next_u64() % range + min
    }

    pub fn next_float(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn next_bool(&mut self, chance: f64) -> bool {
        self.next_float() < chance
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let mut i = slice.len();
        while i > 1 {
            i -= 1;
            let j = self.next_range(0, i as i32 + 1) as usize;
            slice.swap(i, j);
        }
    }

    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let idx = self.next_range(0, slice.len() as i32) as usize;
        Some(&slice[idx])
    }

    pub fn choose_weighted<'a, T>(&mut self, items: &'a [(T, f64)]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        let total: f64 = items.iter().map(|(_, w)| w).sum();
        if total <= 0.0 {
            return None;
        }
        let mut roll = self.next_float() * total;
        for (item, weight) in items {
            roll -= weight;
            if roll <= 0.0 {
                return Some(item);
            }
        }
        items.last().map(|(item, _)| item)
    }

    pub fn choose_weighted_idx(&mut self, weights: &[f64]) -> Option<usize> {
        if weights.is_empty() {
            return None;
        }
        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            return None;
        }
        let mut roll = self.next_float() * total;
        for (i, weight) in weights.iter().enumerate() {
            roll -= weight;
            if roll <= 0.0 {
                return Some(i);
            }
        }
        Some(weights.len() - 1)
    }

    pub fn fork(&mut self) -> GameRng {
        let seed = self.next_u64();
        GameRng::from_seed(seed)
    }

    pub fn seed_snapshot(&self) -> [u64; 4] {
        self.s
    }

    pub fn from_snapshot(s: [u64; 4]) -> Self {
        GameRng { s }
    }
}

fn rotl(x: u64, k: u32) -> u64 {
    x.rotate_left(k)
}

fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

pub enum Distribution {
    Uniform,
    Normal { mean: f64, std_dev: f64 },
    Triangle { min: f64, max: f64, mode: f64 },
}

impl Distribution {
    pub fn sample(&self, rng: &mut GameRng) -> f64 {
        match self {
            Distribution::Uniform => rng.next_float(),
            Distribution::Normal { mean, std_dev } => {
                let u1 = rng.next_float().max(1e-10);
                let u2 = rng.next_float();
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                mean + std_dev * z0
            }
            Distribution::Triangle { min, max, mode } => {
                let u = rng.next_float();
                let range = max - min;
                let mode_ratio = (mode - min) / range;
                if u < mode_ratio {
                    min + range * (u * mode_ratio).sqrt()
                } else {
                    max - range * ((1.0 - u) * (1.0 - mode_ratio)).sqrt()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_seed_same_sequence() {
        let mut a = GameRng::from_seed(42);
        let mut b = GameRng::from_seed(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn test_different_seed_different_sequence() {
        let mut a = GameRng::from_seed(42);
        let mut b = GameRng::from_seed(43);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn test_range() {
        let mut rng = GameRng::from_seed(42);
        for _ in 0..1000 {
            let v = rng.next_range(10, 20);
            assert!(v >= 10 && v < 20);
        }
    }

    #[test]
    fn test_float_range() {
        let mut rng = GameRng::from_seed(42);
        for _ in 0..1000 {
            let v = rng.next_float();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn test_bool_probability() {
        let mut rng = GameRng::from_seed(42);
        let mut true_count = 0;
        let trials = 10000;
        for _ in 0..trials {
            if rng.next_bool(0.3) {
                true_count += 1;
            }
        }
        let ratio = true_count as f64 / trials as f64;
        assert!((ratio - 0.3).abs() < 0.05);
    }

    #[test]
    fn test_shuffle() {
        let mut rng = GameRng::from_seed(42);
        let mut arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let original = arr;
        rng.shuffle(&mut arr);
        assert_ne!(arr, original);
        let mut sorted = arr;
        sorted.sort();
        assert_eq!(sorted, original);
    }

    #[test]
    fn test_choose_weighted() {
        let mut rng = GameRng::from_seed(42);
        let items = [("a", 1.0), ("b", 3.0), ("c", 6.0)];
        let mut counts = [0usize; 3];
        let trials = 10000;
        for _ in 0..trials {
            let chosen = rng.choose_weighted(&items).unwrap();
            match *chosen {
                "a" => counts[0] += 1,
                "b" => counts[1] += 1,
                "c" => counts[2] += 1,
                _ => {}
            }
        }
        assert!(counts[2] > counts[1]);
        assert!(counts[1] > counts[0]);
    }

    #[test]
    fn test_fork_independence() {
        let mut rng = GameRng::from_seed(42);
        let mut child = rng.fork();
        let a = rng.next_u64();
        let b = child.next_u64();
        assert_ne!(a, b);
    }

    #[test]
    fn test_snapshot_restore() {
        let mut rng = GameRng::from_seed(42);
        rng.next_u64();
        rng.next_u64();
        let snapshot = rng.seed_snapshot();
        let v1 = rng.next_u64();
        let mut restored = GameRng::from_snapshot(snapshot);
        let v2 = restored.next_u64();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_normal_distribution() {
        let mut rng = GameRng::from_seed(42);
        let dist = Distribution::Normal { mean: 50.0, std_dev: 10.0 };
        let mut sum = 0.0;
        let n = 10000;
        for _ in 0..n {
            sum += dist.sample(&mut rng);
        }
        let mean = sum / n as f64;
        assert!((mean - 50.0).abs() < 1.0);
    }
}

// Dealer.exe was compiled with 64-bit long int on x86_64
// The LCG overflows into 64-bit space, creating the unique output pattern
pub struct GnuRandom {
    state: [i64; 31],
    fptr: usize,
    rptr: usize,
}

impl GnuRandom {
    pub fn srandom(&mut self, seed: u32) {
        self.state[0] = seed as i64;

        // Dealer.exe uses non-standard constant: 1103515145 (not 1103515245)
        // Source: https://github.com/ThorvaldAagaard/Dealer/blob/main/__random.c
        // The 64-bit LCG overflows, creating negative values and unique output
        for i in 1..31 {
            self.state[i] = self.state[i - 1].wrapping_mul(1103515145).wrapping_add(12345);
        }

        self.fptr = 3;  // SEP_3
        self.rptr = 0;

        // Warmup: 10 * rand_deg iterations
        for _ in 0..(10 * 31) {
            self.warmup_iteration();
        }
    }

    fn warmup_iteration(&mut self) {
        self.state[self.fptr] = self.state[self.fptr].wrapping_add(self.state[self.rptr]);

        // Advance pointers
        self.fptr += 1;
        if self.fptr >= 31 {
            self.fptr = 0;
            self.rptr += 1;
        } else {
            self.rptr += 1;
            if self.rptr >= 31 {
                self.rptr = 0;
            }
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        // Add rear to front (64-bit addition with wrapping)
        self.state[self.fptr] = self.state[self.fptr].wrapping_add(self.state[self.rptr]);

        // 64-bit arithmetic shift right by 1, then mask with 64-bit LONG_MAX
        // This matches the x86_64 assembly: sarq $1, %rax; and $0x7fffffffffffffff, %rax
        let result = (self.state[self.fptr] >> 1) & 0x7fffffffffffffff;

        // Advance pointers
        self.fptr += 1;
        if self.fptr >= 31 {
            self.fptr = 0;
            self.rptr += 1;
        } else {
            self.rptr += 1;
            if self.rptr >= 31 {
                self.rptr = 0;
            }
        }

        // Truncate to 32-bit (as the original returns long but gets cast to unsigned)
        result as u32
    }

    pub fn new() -> Self {
        Self {
            state: [0; 31],
            fptr: 0,
            rptr: 0,
        }
    }
}

impl Default for GnuRandom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_1_first_20_outputs() {
        let mut rng = GnuRandom::new();
        rng.srandom(1);

        // Verified against dealer.exe's rand.o/srand.o using rng_probe
        // Reference: https://www.csc.kth.se/~mdj/guile-ref/guile-ref_69.html (shows first value 269167349)
        let expected: [u32; 20] = [
            269167349,
            3317012772,
            3037285189,
            3401557626,
            2521781105,
            2065258565,
            1482041942,
            628309313,
            1207992583,
            2382384936,
            1768143021,
            3682773873,
            3955356955,
            3180623894,
            3111145845,
            1145084505,
            2396622951,
            3748706040,
            2988814062,
            146139516,
        ];

        for (i, &expected_val) in expected.iter().enumerate() {
            let actual = rng.next_u32();
            assert_eq!(
                actual, expected_val,
                "Mismatch at index {}: expected {}, got {}",
                i, expected_val, actual
            );
        }
    }

    #[test]
    fn test_first_value_matches() {
        // At least verify the first value matches dealer.exe
        let mut rng = GnuRandom::new();
        rng.srandom(1);
        assert_eq!(rng.next_u32(), 269167349);
    }

    #[test]
    fn test_seed_2_first_10_outputs() {
        let mut rng = GnuRandom::new();
        rng.srandom(2);

        // Test with seed=2 to verify different seed works correctly
        // Values from dealer.exe rng_probe with seed=2
        let expected: [u32; 10] = [
            1858980908,
            1463972797,
            3014841053,
            46344911,
            2127386354,
            4256254646,
            2737123461,
            2264856394,
            3087684303,
            1485731095,
        ];

        for (i, &expected_val) in expected.iter().enumerate() {
            let actual = rng.next_u32();
            assert_eq!(
                actual, expected_val,
                "Mismatch at index {} for seed=2: expected {}, got {}",
                i, expected_val, actual
            );
        }
    }
}

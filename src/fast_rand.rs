/*
Fast random numbers, adapted from
https://www.iquilezles.org/www/articles/sfrand/sfrand.htm
*/

use std::num::Wrapping; // rust does't like overflows

union RandCastAssist {
    f: f32,
    u: Wrapping<u32>,
    i: Wrapping<i32>,
}

pub struct FastRand {
    seed: RandCastAssist,
}

impl FastRand {
    pub fn new(seed: u32) -> Self {
        Self {
            seed: RandCastAssist { u: Wrapping(seed) },
        }
    }

    pub fn urand(&mut self) -> f32 {
        unsafe {
            self.seed.i = self.seed.i * Wrapping(16807i32);
            let c = RandCastAssist {
                u: (self.seed.u >> 9) | Wrapping(0x3f800000),
            };
            c.f - 1.0
        }
    }

    pub fn rand(&mut self) -> f32 {
        unsafe {
            self.seed.i = self.seed.i * Wrapping(16807i32);
            let c = RandCastAssist {
                u: (self.seed.u >> 9) | Wrapping(0x40000000),
            };
            c.f - 3.0
        }
    }
}

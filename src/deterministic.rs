//! A copy of the `TestRng` object but it is not stored in a thread local storage and can have multiple instances.
use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64Mcg;

const CL_TEST_SEED: &[u8] = &[
    201, 84, 4, 110, 16, 43, 223, 183, 201, 84, 4, 110, 16, 43, 223, 183,
];

pub struct GenericDeterministicRng<T: SeedableRng>(T);

impl<T: SeedableRng> GenericDeterministicRng<T> {
    fn from_seed(seed: <T as SeedableRng>::Seed) -> Self
    where
        <T as SeedableRng>::Seed: Copy,
    {
        let rng = T::from_seed(seed);
        Self(rng)
    }
}

impl<T> Default for GenericDeterministicRng<T>
where
    T: SeedableRng,
    <T as SeedableRng>::Seed: AsMut<[u8]> + Copy,
{
    fn default() -> Self {
        let mut seed = <<T as SeedableRng>::Seed>::default();
        seed.as_mut().copy_from_slice(CL_TEST_SEED);
        Self::from_seed(seed)
    }
}

impl<T: RngCore + SeedableRng> RngCore for GenericDeterministicRng<T> {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0.try_fill_bytes(dest)
    }
}

pub type DeterministicTestRng = GenericDeterministicRng<Pcg64Mcg>;

#[cfg(test)]
mod tests {
    use std::env;

    use casper_types::testing::TestRng;
    use rand::RngCore;

    use super::*;

    #[test]
    fn should_produce_same_results() {
        env::set_var("CL_TEST_SEED", base16::encode_upper(CL_TEST_SEED));

        let mut rng = DeterministicTestRng::default();
        let mut test_rng = TestRng::new();

        let mut random_bytes_1 = [0u8; 16];
        rng.fill_bytes(&mut random_bytes_1);
        let mut random_bytes_2 = [0u8; 16];
        test_rng.fill_bytes(&mut random_bytes_2);

        assert_eq!(random_bytes_1, random_bytes_2);
    }
}

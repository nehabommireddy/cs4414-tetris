use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_xoshiro::Xoshiro128PlusPlus;

type RngAlgorithm = Xoshiro128PlusPlus;

pub struct BagRandomizer<T> {
    bag: Vec<T>,
    idx: usize,
    rng: RngAlgorithm,
}

impl<T> BagRandomizer<T> {
    pub fn from_parts(
        bag: impl Iterator<Item = T>,
        seed: <RngAlgorithm as SeedableRng>::Seed,
    ) -> Self {
        let mut rng = RngAlgorithm::from_seed(seed);

        let mut bag: Vec<T> = bag.collect();
        bag.shuffle(&mut rng);

        Self { idx: 0, bag, rng }
    }

    pub fn next(&mut self) {
        self.idx += 1;

        if self.idx >= self.bag.len() {
            self.bag.shuffle(&mut self.rng);
            self.idx = 0;
        }
    }

    pub fn cur(&self) -> &T {
        &self.bag[self.idx]
    }
}

use core::cmp::Ordering;

#[derive(Eq, PartialEq, Debug)]
pub struct ScoredValue<T: Eq + PartialEq> {
    pub value: T,
    pub score: usize,
}

impl<T: Eq> Ord for ScoredValue<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl<T: Eq + PartialEq> PartialOrd for ScoredValue<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

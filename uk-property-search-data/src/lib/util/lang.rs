use anyhow::Result;
use itertools::Itertools;

pub trait VecResultExt<T> {
    fn unwrap_all(self) -> Vec<T>;
}

impl<T> VecResultExt<T> for Vec<Result<T>> {
    fn unwrap_all(self) -> Vec<T> {
        self.into_iter().map(|result| result.unwrap()).collect_vec()
    }
}

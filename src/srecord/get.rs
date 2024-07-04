pub trait Get<T> {
    type Output: ?Sized;

    fn get(&self, index: T) -> Option<&Self::Output>;
    fn get_mut(&mut self, index: T) -> Option<&mut Self::Output>;
}

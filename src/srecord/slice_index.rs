/// Trait that helps index into data structures with different index and return types.
///
/// The `get` and `get_mut` methods of [`DataChunk`] and [`SRecordFile`] can be used to optionally
/// get data from their respective data structure, using any indexing type that implements
/// [`SliceIndex`]. Currently, `u64` is used to get the data at a single address, and
/// [`Range<u64>`](`Range`) is used to index a slice of contiguous data.
pub trait SliceIndex<T: ?Sized>: private::Sealed {
    type Output: ?Sized;

    fn get(self, data: &T) -> Option<&Self::Output>;
    fn get_mut(self, data: &mut T) -> Option<&mut Self::Output>;
}

mod private {
    use std::ops::Range;

    pub trait Sealed {}
    impl Sealed for u64 {}
    impl Sealed for Range<u64> {}
}

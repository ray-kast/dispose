/// A helper trait for objects that must be consumed with the help of another value.
///
/// If the method to consume a value is an associated function or requires additional arguments
/// that cannot be determined at compile-time, this trait provides a simple way to implement
/// Dispose for any container that has both the value to be disposed and any other values
/// necessary, as demonstrated with [this implementation for pairs][(W, T)].
///
/// [(W, T)]: ./trait.Dispose.html#impl-Dispose-for-(W%2C%20T)
pub trait DisposeWith<W> {
    /// Dispose self, using the provided value.
    fn dispose_with(self, with: W);
}

impl<W, F: FnOnce(W)> DisposeWith<W> for F {
    /// Run the closure with the given parameter, consuming both.
    fn dispose_with(self, with: W) { self(with); }
}

/// A helper trait for iterators with items implementing `DisposeWith`.
///
/// This trait exists for a reason similar to [`DisposeIterator`] &mdash; namely, that implementing
/// `DisposeWith` for `IntoIterator` causes conflicts with other implementations.  More info can be
/// found in the documentation for [`DisposeIterator`].
///
/// [`DisposeIterator`]: ./trait.DisposeIterator.html
pub trait DisposeIteratorWith<W> {
    /// Dispose all items in the iterator using the provided value, consuming both.
    fn dispose_iter_with(self, with: W);
}

impl<W: Copy, I: IntoIterator> DisposeIteratorWith<W> for I
where I::Item: DisposeWith<W>
{
    /// Dispose all items in the iterator, using copies of the provided value.
    fn dispose_iter_with(self, with: W) {
        for el in self {
            el.dispose_with(with);
        }
    }
}

// TODO: add a more relaxed Clone impl whenever trait specialization is stabilized

impl<W, T> DisposeWith<W> for Vec<T>
where Vec<T>: DisposeIteratorWith<W>
{
    fn dispose_with(self, with: W) { self.dispose_iter_with(with) }
}

impl<W, T> DisposeWith<W> for Box<[T]>
where Vec<T>: DisposeIteratorWith<W>
{
    fn dispose_with(self, with: W) { self.into_vec().dispose_iter_with(with) }
}

impl<'a, W, T> DisposeWith<W> for &'a [T]
where &'a [T]: DisposeIteratorWith<W>
{
    fn dispose_with(self, with: W) { self.dispose_iter_with(with) }
}

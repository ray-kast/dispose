use super::DisposeWith;

/// A trait representing a standard "dispose" method for consuming an object at
/// the end of its scope.
///
/// The typical use case of this trait is for encapsulating objects in
/// [`Disposable`] wrappers, which will automatically call [`dispose`] on drop,
/// but it is perfectly acceptable to call [`dispose`] by itself.
///
/// See [this page][examples] for example usage.
///
/// [`Disposable`]: ./struct.Disposable.html
/// [`dispose`]: ./trait.Dispose.html#method.dispose
/// [examples]: ./index.html#examples
pub trait Dispose {
    /// Consume self and deinitialize its contents.
    fn dispose(self);
}

impl<F: FnOnce()> Dispose for F {
    /// Run the closure, consuming it.
    fn dispose(self) { self() }
}

impl<W, T: DisposeWith<W>> Dispose for (W, T)
where (W, T): Sized
{
    /// Dispose `self.1`, passing `self.0` to `dispose_with`.
    fn dispose(self) { self.1.dispose_with(self.0) }
}

/// A helper trait for iterators with items implementing `Dispose`.
///
/// This trait exists mainly because of the stringency of Rust's trait solver
/// &mdash; implementing `Dispose` for all `I: IntoIterator` with `I::Item:
/// Dispose` causes conflicts with the other blanket implementations of
/// `Dispose`, so this trait serves as a replacement.
///
/// Several types with `DisposeIterator` implementations (such as [`Vec<T>`])
/// have dedicated `Dispose` implementations for convenience.
pub trait DisposeIterator {
    /// Dispose all items in the iterator, consuming it.
    fn dispose_iter(self);
}

impl<I: IntoIterator> DisposeIterator for I
where I::Item: Dispose
{
    fn dispose_iter(self) {
        for el in self {
            el.dispose()
        }
    }
}

impl<T> Dispose for Vec<T>
where Vec<T>: DisposeIterator
{
    fn dispose(self) { self.dispose_iter() }
}

impl<T> Dispose for Box<[T]>
where Vec<T>: DisposeIterator
{
    fn dispose(self) { self.into_vec().dispose_iter() }
}

impl<'a, T> Dispose for &'a [T]
where &'a [T]: DisposeIterator
{
    fn dispose(self) { self.dispose_iter() }
}

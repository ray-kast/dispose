use super::Dispose;
use std::{
    borrow::{Borrow, BorrowMut},
    mem::{forget, ManuallyDrop},
    ops::{Deref, DerefMut},
};

/// Wrapper for values implementing [`Dispose`] that provides a `Drop` implementation.
///
/// This struct will automatically consume its contents on drop using the provided [`Dispose`]
/// implementation.
///
/// See [this page][examples] for example usage.
///
/// [`Dispose`]: ./trait.Dispose.html
/// [examples]: ./index.html#examples
#[derive(Debug)]
pub struct Disposable<T: Dispose>(ManuallyDrop<T>);

impl<T: Dispose> Disposable<T> {
    /// Construct a new `Disposable` instance, wrapping around `val`.
    pub fn new(val: T) -> Self { Self(ManuallyDrop::new(val)) }

    /// Consume the wrapper, producing the contained value.
    ///
    /// # Safety
    /// 
    /// It is up to the user to ensure the value does not fall out of scope without being consumed.
    /// 
    /// The value can be safely re-inserted into a `Disposable` using `Disposable::new` to restore
    /// safe drop behavior, and it is recommended that the value is held by some container which
    /// consumes it on drop at all times.  The intended use case for this function is transferring
    /// the value from one container to the other.
    pub unsafe fn leak(mut this: Self) -> T {
        let inner = ManuallyDrop::take(&mut this.0);
        forget(this);
        inner
    }
}

impl<T: Dispose> From<T> for Disposable<T> {
    fn from(val: T) -> Self { Self::new(val) }
}

impl<T: Dispose> Drop for Disposable<T> {
    fn drop(&mut self) {
        let inner = unsafe { ManuallyDrop::take(&mut self.0) };

        inner.dispose();
    }
}

impl<T: Dispose> AsRef<T> for Disposable<T> {
    fn as_ref(&self) -> &T { &self.0 }
}

impl<T: Dispose> AsMut<T> for Disposable<T> {
    fn as_mut(&mut self) -> &mut T { &mut self.0 }
}

impl<T: Dispose> Borrow<T> for Disposable<T> {
    fn borrow(&self) -> &T { self.as_ref() }
}

impl<T: Dispose> BorrowMut<T> for Disposable<T> {
    fn borrow_mut(&mut self) -> &mut T { self.as_mut() }
}

impl<T: Dispose> Deref for Disposable<T> {
    type Target = T;

    fn deref(&self) -> &T { self.as_ref() }
}

impl<T: Dispose> DerefMut for Disposable<T> {
    fn deref_mut(&mut self) -> &mut T { self.as_mut() }
}

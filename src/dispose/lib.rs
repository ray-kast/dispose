#![warn(missing_docs)]

//! A small crate for handling resources that must be consumed at the end of their lifetime.
//!
//! Since Rust's type system is affine rather than linear, `Drop::drop` mutably borrows `self`,
//! rather than consuming it.  For the most part, this is fine, but for some cases (such as working
//! with the crate `gfx_hal`) resources must be consumed on drop.  This crate and the
//! `dispose_derive` crate serve to cover the typical boilerplate for such cases by managing the
//! `ManuallyDrop` wrapper for you.
//!
//! As a bonus, this crate makes it easy to defer the execution of an `FnOnce` closure to the end
//! of a scope, which can be done using the [`defer`] function.
//!
//! # Examples
//!
//! ```rust
//! use dispose::{Dispose, Disposable};
//!
//! struct MyStruct;
//!
//! impl Dispose for MyStruct {
//!     fn dispose(self) { println!("Goodbye, world!"); }
//! }
//!
//! {
//!     let _my_struct = Disposable::new(MyStruct);
//! } // prints "Goodbye, world!"
//! ```
//!
//! [`defer`]: ./fn.defer.html

use std::{
    borrow::{Borrow, BorrowMut},
    mem,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

pub use dispose_derive::*;

/// A trait representing a standard "dispose" method for consuming an object at the end of its
/// scope.
///
/// The typical use case of this trait is for encapsulating objects in [`Disposable`] wrappers,
/// which will automatically call [`dispose`] on drop, but it is perfectly acceptable to call
/// [`dispose`] by itself.
///
/// See [this page][examples] for example usage.
///
/// [`Disposable`]: ./struct.Disposable.html
/// [`dispose`]: ./trait.Dispose.html#tymethod.dispose
/// [examples]: ./index.html#examples
pub trait Dispose {
    /// Consume self and deinitialize its contents.
    fn dispose(self);
}

/// Wrapper for values implementing [`Dispose`] that provides a `Drop` implementation.
///
/// This struct will automatically consume its contents on drop using the provided [`Dispose`]
/// implementation.
///
/// See [this page][examples] for example usage.
///
/// [`Dispose`]: ./trait.Dispose.html
/// [examples]: ./index.html#examples
pub struct Disposable<T: Dispose>(ManuallyDrop<T>);

impl<T: Dispose> Disposable<T> {
    /// Construct a new `Disposable` instance, wrapping around `val`.
    pub fn new(val: T) -> Self { Self(ManuallyDrop::new(val)) }

    /// Consume the wrapper, producing the contained value.
    ///
    /// **NOTE:** It is up to the user to ensure the value gets consumed once it is leaked.
    pub unsafe fn leak(mut this: Self) -> T {
        let inner = ManuallyDrop::take(&mut this.0);
        mem::forget(this);
        inner
    }
}

impl<T: Dispose> From<T> for Disposable<T> {
    fn from(val: T) -> Self { Self::new(val) }
}

impl<T: Dispose> Drop for Disposable<T> {
    fn drop(&mut self) {
        let disp = unsafe { ManuallyDrop::take(&mut self.0) };

        disp.dispose();
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

impl<F: FnOnce()> Dispose for F {
    /// Run the closure, consuming it.
    fn dispose(self) { self() }
}

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

impl<W, T: DisposeWith<W>> Dispose for (W, T) {
    /// Dispose `self.1`, passing `self.0` to `dispose_with`.
    fn dispose(self) { self.1.dispose_with(self.0) }
}

impl<W, F: FnOnce(W)> DisposeWith<W> for F {
    /// Run the closure with the provided argument, consuming the closure.
    fn dispose_with(self, with: W) { self(with) }
}

/// Defer an action until the end of a lexical scope.
///
/// This function returns a value that calls the provided closure when dropped, resulting in
/// functionality akin to `try...finally` blocks or Swift's `defer` blocks and Go's `defer func`.
///
/// # Examples
///
/// ```rust
/// fn main() {
///     let _d = defer(|| println!("Hello from defer()!"));
///
///     println!("Hello, world!");
/// }
///
/// // This prints the following:
/// // Hello, world!
/// // Hello from defer()!
/// ```
///
/// A more pertinent example would be the use of defer with the `?` operator:
///
/// ```rust
/// fn try_me() -> Result<(), ()> {
///     let _d = defer(|| println!("Cleaning up..."));
///
///     println!("Hello!");
///
///     let uh_oh = Err(())?; // Pretend this was a function that failed
///
///     println!("You can't see me: {:?}", uh_oh);
/// }
///
/// try_me();
///
/// // This prints the following:
/// // Hello!
/// // Cleaning up...
pub fn defer<F: FnOnce()>(f: F) -> Disposable<F> { f.into() }

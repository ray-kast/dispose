#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::cargo)]
#![deny(intra_doc_link_resolution_failure, missing_debug_implementations)]

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
//! **NOTE:** The `Dispose` trait does _not_ provide a `Drop` impl by itself.  For that, a value
//! implementing `Dispose` must be wrapped in a [`Disposable`] struct.
//!
//! # Examples
//!
//! ```
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
//! As a design consideration, values implementing `Dispose` should _always_ be returned from
//! functions within `Disposable` or any other wrapper properly implementing `Drop`.  `Disposable`
//! is recommended as it contains an unsafe [`leak`] function to retrieve the inner value, if
//! necessary.
//!
//! ```
//! use dispose::{Dispose, Disposable};
//!
//! mod secrets {
//! #   use dispose::{Dispose, Disposable};
//!
//!     pub struct Secrets {
//!         launch_codes: u32,
//!     }
//!
//!     impl Secrets {
//!         pub fn new(launch_codes: u32) -> Disposable<Self> {
//!             Self { launch_codes }.into()
//!         }
//!     }
//!
//!     impl Dispose for Secrets {
//!         fn dispose(mut self) { self.launch_codes = 0x0; } // Nice try, hackers!
//!     }
//! }
//!
//! fn main() {
//!     let secret = secrets::Secrets::new(0xDEADBEEF);
//! } // secret is properly disposed at the end of the scope
//!
//! fn BAD() {
//!     let secret = secrets::Secrets::new(0o1337);
//!
//!     let mwahaha = unsafe { Disposable::leak(secret) };
//! } // .dispose() was not called - data has been leaked!
//! ```
//!
//! (My lawyers have advised me to note that the above example is not cryptographically secure.
//! Please do not clean up secure memory by simply setting it to zero.)
//!
//! [`defer`]: ./fn.defer.html
//! [`Disposable`]: ./struct.Disposable.html
//! [`leak`]: ./struct.Disposable.html#tymethod.leak

mod defer;
mod disposable;
mod dispose;
mod dispose_with;

pub use crate::{defer::*, disposable::*, dispose::*, dispose_with::*};
pub use dispose_derive::*;

/// Contains all the basic traits and derive macros exported by this crate.
pub mod prelude {
    #[doc(no_inline)]
    pub use super::{Dispose, DisposeWith};
    #[doc(no_inline)]
    pub use dispose_derive::*;
}

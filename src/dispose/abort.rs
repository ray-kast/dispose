#![allow(clippy::module_name_repetitions)]

use std::process::abort;

use crate::{Disposable, Dispose};

/// Abort the process if this value is dropped.
///
/// This struct is for bypassing an unwinding panic should something go horribly
/// wrong in an application.  It is the heart of [`abort_on_panic`], and for
/// most cases does not need to be used directly.
///
/// [`abort_on_panic`]: ./function.abort_on_panic.html
#[derive(Debug)]
pub struct AbortCanary(());

impl AbortCanary {
    /// Construct a new canary.
    ///
    /// # Panics
    /// The value produced by this function must be passed to [`release`] in
    /// order to avoid aborting the process.
    ///
    /// [`release`]: ./struct.AbortCanary.html#method.release
    #[must_use = "Dropping this value immediately will abort the process."]
    pub fn new() -> Disposable<Self> { Disposable::new(Self(())) }

    /// Release `canary`.  This will consume and drop it without aborting the
    /// process.
    pub fn release(canary: Disposable<Self>) { unsafe { Disposable::leak(canary) }; }
}

impl Dispose for AbortCanary {
    fn dispose(self) { abort(); }
}

/// Abort the process if the provided closure panics.
///
/// This function internally constructs an [`AbortCanary`] and releases it after
/// running `f`.
///
/// # Example
/// The following panic will result in the process aborting:
///
/// ```should_panic
/// # use dispose::abort_on_panic;
/// abort_on_panic(|| panic!("oops!"));
/// ```
///
/// [`AbortCanary`]: ./struct.AbortCanary.html
pub fn abort_on_panic<T>(f: impl FnOnce() -> T) -> T {
    let canary = AbortCanary::new();

    let ret = f();

    AbortCanary::release(canary);

    ret
}

#[cfg(test)]
mod test {
    use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn safe_canary() {
        let canary = AbortCanary::new();
        AbortCanary::release(canary);
    }

    // TODO: how to test abort???
    //       I have confirmed that uncommenting the commented tests does result
    //       in the test process aborting, but currently the test runner can't
    //       deal with that properly.

    // #[test]
    // #[should_panic]
    // fn bad_canary() { let _canary = AbortCanary::new(); }

    #[test]
    fn safe_aop() { abort_on_panic(|| ()); }

    // This should NOT panic
    #[test]
    fn sanity_check_aop() { catch_unwind(|| panic!()); }

    // // This should panic
    // #[test]
    // #[should_panic]
    // fn bad_aop() { catch_unwind(|| abort_on_panic(|| panic!())).ok(); }
}

use super::Disposable;

/// Defer an action until the end of a lexical scope.
///
/// This function returns a value that calls the provided closure when dropped, resulting in
/// functionality akin to `try...finally` blocks or Swift's `defer` blocks and Go's `defer func`.
///
/// # Examples
///
/// ```
/// use dispose::defer;
///
/// {
///     let _d = defer(|| println!("Hello from defer()!"));
///
///     println!("Hello, world!");
/// }
/// // This prints the following:
/// // Hello, world!
/// // Hello from defer()!
/// ```
///
/// A more pertinent example would be the use of defer with the `?` operator:
///
/// ```
/// use dispose::defer;
///
/// fn tryme() -> Result<(), ()> {
///     let _d = defer(|| println!("Cleaning up..."));
///
///     println!("Hello!");
///
///     let uh_oh = Err(())?; // Pretend this was a function that failed
///
///     println!("You can't see me: {:?}", uh_oh);
/// 
///     Ok(())
/// }
/// 
/// println!("hi");
/// 
/// tryme().map_err(|()| println!("ERROR: Something went wrong.")).unwrap_err()
///
/// // This prints the following:
/// // Hello!
/// // Cleaning up...
/// // ERROR: Something went wrong.
/// ```
pub fn defer<F: FnOnce()>(f: F) -> Disposable<F> { f.into() }

/// Defer an action until the end of a lexical scope, passing the provided argument; similar to
/// [`defer`].
///
/// This function is mainly provided for completeness, and is less useful than using [`FnOnce(W)`
/// as `DisposeWith<W>`][`FnOnce(W)`] for providing a struct with dynamic teardown logic with a
/// provided value.
///
/// [`defer`]: ./fn.defer.html
/// [`FnOnce(W)`]: ./trait.DisposeWith.html#impl-DisposeWith<W>
pub fn defer_with<W, F: FnOnce(W)>(with: W, f: F) -> Disposable<(W, F)> { (with, f.into()).into() }

use std::borrow::Cow;

// --- Version 1: Returning &str (Simpler, good for static defaults) ---

/// Extension trait for string types to provide a default for empty strings.
pub trait StringExt {
    /// Returns the string slice if it's not empty, otherwise computes a default
    /// from the provided closure.
    ///
    /// The lifetime of the returned `&str` is tied to the lifetime of `self`
    /// or the lifetime of the string returned by `default_fn`.
    /// For this to compile, the string slice returned by `default_fn` must live
    /// at least as long as the borrow of `self`. This is typically true for
    /// string literals (which are `&'static str`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate_name::StringExt; // Assuming you put this in your_crate_name
    /// let s1 = "hello";
    /// assert_eq!(s1.unwrap_or_default(|| "default"), "hello");
    ///
    /// let s2 = "";
    /// assert_eq!(s2.unwrap_or_default(|| "default from closure"), "default from closure");
    ///
    /// let my_string = String::from("");
    /// assert_eq!(my_string.unwrap_or_default(|| "another default"), "another default");
    ///
    /// let non_empty_string = String::from("world");
    /// assert_eq!(non_empty_string.unwrap_or_default(|| "wont be used"), "world");
    /// ```
    fn unwrap_or_default<'a, F>(&'a self, default_fn: F) -> &'a str
    where
        F: FnOnce() -> &'a str;
}

// Implement the trait for `str`.
// This will cover `&str` and also `String` (due to `Deref<Target=str>`).
impl StringExt for str {
    fn unwrap_or_default<'a, F>(&'a self, default_fn: F) -> &'a str
    where
        F: FnOnce() -> &'a str,
    {
        if self.is_empty() {
            default_fn()
        } else {
            self
        }
    }
}

use const_panic::concat_panic;
use konst::eq_str;

/// Check if a static array contains a key
const fn contains(arr: &[&str], key: &str) -> bool {
    let mut idx = 0;
    while idx < arr.len() {
        if eq_str(arr[idx], key) {
            return true;
        }
        idx += 1;
    }
    false
}

/// Asserts that an array contains all items in a different array
///
/// This is used to assert that derived shapes contain ALL keys from
/// their parent schema.
///
/// ### Example
///
/// ```
/// use smithy4rs_core::assert_contains_all;
///
/// const KEYS: &[&str] = &["a", "b", "c"];
///
/// const _: () = assert_contains_all(KEYS, &["a", "b", "c"]);
/// ```
pub const fn assert_contains_all(this: &[&str], other: &[&str]) {
    let mut idx = 0;
    while idx < this.len() {
        if !contains(this, other[idx]) {
            concat_panic!(
                "unexpected member: `",
                other[idx],
                "`. Expected one of: ",
                this
            );
        }
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{assert_contains_all, contains};

    const KEYS: &[&str] = &["foo", "bar", "baz"];

    #[test]
    fn test_contains() {
        assert!(contains(KEYS, "bar"));
        assert!(contains(KEYS, "foo"));
        assert!(!contains(KEYS, "quux"));
    }

    #[test]
    fn test_contains_all() {
        assert_contains_all(KEYS, &["foo", "bar", "baz"]);
    }

    #[test]
    #[should_panic(expected = "unexpected member:")]
    fn test_not_contains() {
        assert_contains_all(KEYS, &["foo", "quux"]);
    }
}

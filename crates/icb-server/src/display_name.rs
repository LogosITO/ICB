//! Conversions from Clang USR strings to display-friendly identifiers.
//!
//! A USR (Unified Symbol Resolution) uniquely identifies a declaration in the
//! Clang AST. Its structure encodes namespaces, classes, function signatures
//! and template parameters. This module extracts the last, human-significant
//! segment of the USR and strips template suffix clutter.

/// Returns a readable name from a raw identifier that may be a Clang USR.
///
/// # USR format
///
/// USRs follow the pattern `c:…@…@name#…`.  The last `@`-delimited segment
/// contains the symbol’s base name, optionally followed by `#…` encoding
/// template arguments and qualifiers.
///
/// # Strategy
///
/// 1. If `raw` does **not** contain `@`, it is already a plain name and is
///    returned unchanged.
/// 2. Otherwise, the substring after the last `@` is taken.
/// 3. Everything from the first `#` in that substring is discarded.
///
/// # Examples
///
/// ```rust
/// # use icb_server::display_name::readable_name;
/// assert_eq!(readable_name("c:@F@main"), "main");
/// assert_eq!(readable_name("c:@S@MyClass@F@method"), "method");
/// assert_eq!(
///     readable_name("c:@S@MyClass@F@MyClass#&1$@S@MyClass#"),
///     "MyClass"
/// );
/// assert_eq!(readable_name("already_clean"), "already_clean");
/// assert_eq!(readable_name(""), "");
/// ```
pub fn readable_name(raw: &str) -> String {
    if !raw.contains('@') {
        return raw.to_string();
    }

    let after_at = raw.rsplit('@').next().unwrap_or(raw);
    let before_hash = after_at.split('#').next().unwrap_or(after_at);
    before_hash.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        assert_eq!(readable_name("c:@F@main"), "main");
    }

    #[test]
    fn test_class_method() {
        assert_eq!(readable_name("c:@S@MyClass@F@method"), "method");
    }

    #[test]
    fn test_constructor_template() {
        assert_eq!(
            readable_name("c:@S@MyClass@F@MyClass#&1$@S@MyClass#"),
            "MyClass"
        );
    }

    #[test]
    fn test_already_clean() {
        assert_eq!(readable_name("helper"), "helper");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(readable_name(""), "");
    }

    #[test]
    fn test_only_at() {
        assert_eq!(readable_name("@"), "");
    }
}

//! Identifier validation types.
//!
//! Provides validated types for common identifier formats like URL slugs,
//! usernames, and product SKUs. These types encode validity at the type level—
//! once you have a `Slug`, it's guaranteed to be a valid URL path component.
//!
//! # Philosophy
//!
//! This module follows the Stillwater philosophy:
//! - **Parse, Don't Validate**: `Slug` encodes URL-safety at the type level
//! - **Errors Should Tell Stories**: Errors identify invalid characters and positions
//! - **Composition Over Complexity**: Simple predicates for character validation
//! - **Pragmatism Over Purity**: `from_title()` converts prose to slugs for convenience
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "identifiers")]
//! # {
//! use stilltypes::identifiers::{Slug, SlugExt};
//!
//! // Validate existing slug
//! let slug = Slug::new("my-first-post".to_string()).unwrap();
//! assert_eq!(slug.get(), "my-first-post");
//!
//! // Convert from title
//! let slug = Slug::from_title("My First Blog Post!").unwrap();
//! assert_eq!(slug.get(), "my-first-blog-post");
//! # }
//! ```

use crate::error::{DomainError, DomainErrorKind};
use stillwater::refined::{Predicate, Refined};

/// Maximum length for a slug (reasonable for URLs).
const MAX_SLUG_LENGTH: usize = 128;

// ============================================================================
// Slug
// ============================================================================

/// Predicate for valid URL slugs.
///
/// Validates strings matching the pattern `^[a-z0-9]+(-[a-z0-9]+)*$`:
/// - Only lowercase letters, digits, and hyphens allowed
/// - Must start with a letter or digit
/// - Must end with a letter or digit
/// - No consecutive hyphens (`--`)
/// - Length: 1-128 characters
///
/// # Example
///
/// ```
/// # #[cfg(feature = "identifiers")]
/// # {
/// use stilltypes::identifiers::Slug;
///
/// let slug = Slug::new("my-first-post".to_string());
/// assert!(slug.is_ok());
///
/// let invalid = Slug::new("My First Post".to_string());
/// assert!(invalid.is_err());
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidSlug;

impl Predicate<String> for ValidSlug {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        // Empty check
        if value.is_empty() {
            return Err(DomainError {
                format_name: "slug",
                value: value.clone(),
                reason: DomainErrorKind::Empty,
                example: "my-first-post",
            });
        }

        // Length check
        if value.len() > MAX_SLUG_LENGTH {
            return Err(DomainError {
                format_name: "slug",
                value: format!("{}...", &value[..32.min(value.len())]),
                reason: DomainErrorKind::TooLong {
                    max: MAX_SLUG_LENGTH,
                    actual: value.len(),
                },
                example: "my-first-post",
            });
        }

        let chars: Vec<char> = value.chars().collect();

        // Check first character - must be alphanumeric
        if !chars[0].is_ascii_lowercase() && !chars[0].is_ascii_digit() {
            return Err(DomainError {
                format_name: "slug",
                value: value.clone(),
                reason: DomainErrorKind::InvalidCharacter {
                    char: chars[0],
                    position: 0,
                },
                example: "my-first-post",
            });
        }

        // Check last character - must be alphanumeric
        let last = chars.len() - 1;
        if !chars[last].is_ascii_lowercase() && !chars[last].is_ascii_digit() {
            return Err(DomainError {
                format_name: "slug",
                value: value.clone(),
                reason: DomainErrorKind::InvalidCharacter {
                    char: chars[last],
                    position: last,
                },
                example: "my-first-post",
            });
        }

        // Check all characters and consecutive hyphens
        let mut prev_hyphen = false;
        for (i, c) in chars.iter().enumerate() {
            if *c == '-' {
                if prev_hyphen {
                    return Err(DomainError {
                        format_name: "slug",
                        value: value.clone(),
                        reason: DomainErrorKind::InvalidFormat {
                            expected: "no consecutive hyphens",
                        },
                        example: "my-first-post",
                    });
                }
                prev_hyphen = true;
            } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
                prev_hyphen = false;
            } else {
                return Err(DomainError {
                    format_name: "slug",
                    value: value.clone(),
                    reason: DomainErrorKind::InvalidCharacter {
                        char: *c,
                        position: i,
                    },
                    example: "my-first-post",
                });
            }
        }

        Ok(())
    }

    fn description() -> &'static str {
        "URL-safe slug (lowercase alphanumeric with hyphens)"
    }
}

/// A validated URL-safe slug.
///
/// A `String` that has been validated to be a properly formatted slug:
/// - Only lowercase letters, digits, and hyphens
/// - Starts and ends with alphanumeric characters
/// - No consecutive hyphens
/// - 1-128 characters
///
/// # Example
///
/// ```
/// # #[cfg(feature = "identifiers")]
/// # {
/// use stilltypes::identifiers::Slug;
///
/// let slug = Slug::new("my-first-post".to_string()).unwrap();
/// assert_eq!(slug.get(), "my-first-post");
/// # }
/// ```
pub type Slug = Refined<String, ValidSlug>;

/// Extension trait for slug operations.
///
/// Provides helper methods for working with slugs, including conversion
/// from prose titles.
pub trait SlugExt {
    /// Convert a title or prose string into a valid slug.
    ///
    /// Performs the following transformations:
    /// - Converts to lowercase
    /// - Replaces spaces, underscores, and non-alphanumeric chars with hyphens
    /// - Collapses consecutive hyphens into single hyphens
    /// - Trims leading and trailing hyphens
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "identifiers")]
    /// # {
    /// use stilltypes::identifiers::{Slug, SlugExt};
    ///
    /// let slug = Slug::from_title("My First Blog Post!").unwrap();
    /// assert_eq!(slug.get(), "my-first-blog-post");
    ///
    /// let slug = Slug::from_title("  Spaces  Everywhere  ").unwrap();
    /// assert_eq!(slug.get(), "spaces-everywhere");
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the resulting slug would be empty (e.g., input
    /// contains only special characters).
    fn from_title(title: &str) -> Result<Slug, DomainError>;
}

impl SlugExt for Slug {
    fn from_title(title: &str) -> Result<Slug, DomainError> {
        // Convert to lowercase and replace non-alphanumeric with hyphens
        let slug: String = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>()
            // Split by hyphens, filter empty parts, rejoin
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        if slug.is_empty() {
            return Err(DomainError {
                format_name: "slug",
                value: title.to_string(),
                reason: DomainErrorKind::Empty,
                example: "my-first-post",
            });
        }

        Slug::new(slug)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Slug Validation Tests
    // ========================================================================

    mod valid_slugs {
        use super::*;

        #[test]
        fn single_letter() {
            assert!(Slug::new("a".to_string()).is_ok());
        }

        #[test]
        fn single_digit() {
            assert!(Slug::new("1".to_string()).is_ok());
        }

        #[test]
        fn simple_word() {
            assert!(Slug::new("hello".to_string()).is_ok());
        }

        #[test]
        fn hyphenated() {
            assert!(Slug::new("hello-world".to_string()).is_ok());
        }

        #[test]
        fn with_numbers() {
            assert!(Slug::new("post-123".to_string()).is_ok());
        }

        #[test]
        fn multiple_segments() {
            assert!(Slug::new("a1-b2-c3".to_string()).is_ok());
        }

        #[test]
        fn starts_with_digit() {
            assert!(Slug::new("123-numbers-first".to_string()).is_ok());
        }

        #[test]
        fn ends_with_digit() {
            assert!(Slug::new("chapter-1".to_string()).is_ok());
        }

        #[test]
        fn max_length() {
            let slug = "a".repeat(128);
            assert!(Slug::new(slug).is_ok());
        }

        #[test]
        fn mixed_alphanumeric_segments() {
            assert!(Slug::new("v1-alpha-2-beta".to_string()).is_ok());
        }
    }

    mod invalid_slugs {
        use super::*;

        #[test]
        fn empty() {
            let result = Slug::new(String::new());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::Empty));
        }

        #[test]
        fn uppercase() {
            let result = Slug::new("Hello".to_string());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidCharacter {
                    char: 'H',
                    position: 0
                }
            ));
        }

        #[test]
        fn all_uppercase() {
            let result = Slug::new("HELLO".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn spaces() {
            let result = Slug::new("hello world".to_string());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidCharacter {
                    char: ' ',
                    position: 5
                }
            ));
        }

        #[test]
        fn special_characters() {
            let result = Slug::new("hello@world".to_string());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidCharacter {
                    char: '@',
                    position: 5
                }
            ));
        }

        #[test]
        fn dot() {
            let result = Slug::new("hello.world".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn consecutive_hyphens() {
            let result = Slug::new("hello--world".to_string());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidFormat {
                    expected: "no consecutive hyphens"
                }
            ));
        }

        #[test]
        fn triple_hyphens() {
            let result = Slug::new("a---b".to_string());
            assert!(result.is_err());
        }

        #[test]
        fn leading_hyphen() {
            let result = Slug::new("-hello".to_string());
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidCharacter {
                    char: '-',
                    position: 0
                }
            ));
        }

        #[test]
        fn trailing_hyphen() {
            let result = Slug::new("hello-".to_string());
            assert!(result.is_err());
            let err = result.unwrap_err();
            let len = "hello-".len() - 1;
            assert!(matches!(
                err.reason,
                DomainErrorKind::InvalidCharacter { char: '-', position: p } if p == len
            ));
        }

        #[test]
        fn too_long() {
            let slug = "a".repeat(129);
            let result = Slug::new(slug);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(
                err.reason,
                DomainErrorKind::TooLong {
                    max: 128,
                    actual: 129
                }
            ));
        }

        #[test]
        fn underscore() {
            let result = Slug::new("hello_world".to_string());
            assert!(result.is_err());
        }
    }

    // ========================================================================
    // from_title() Tests
    // ========================================================================

    mod from_title {
        use super::*;

        #[test]
        fn simple_title() {
            let slug = Slug::from_title("My First Post").unwrap();
            assert_eq!(slug.get(), "my-first-post");
        }

        #[test]
        fn with_extra_spaces() {
            let slug = Slug::from_title("  Spaces  Everywhere  ").unwrap();
            assert_eq!(slug.get(), "spaces-everywhere");
        }

        #[test]
        fn with_special_characters() {
            let slug = Slug::from_title("Special!@#Characters").unwrap();
            assert_eq!(slug.get(), "special-characters");
        }

        #[test]
        fn already_a_slug() {
            let slug = Slug::from_title("already-a-slug").unwrap();
            assert_eq!(slug.get(), "already-a-slug");
        }

        #[test]
        fn mixed_case_slug() {
            let slug = Slug::from_title("Already-A-Slug").unwrap();
            assert_eq!(slug.get(), "already-a-slug");
        }

        #[test]
        fn numbers_first() {
            let slug = Slug::from_title("123 Numbers First").unwrap();
            assert_eq!(slug.get(), "123-numbers-first");
        }

        #[test]
        fn with_apostrophe() {
            let slug = Slug::from_title("It's a Test").unwrap();
            assert_eq!(slug.get(), "it-s-a-test");
        }

        #[test]
        fn with_punctuation() {
            let slug = Slug::from_title("Hello, World! How's it going?").unwrap();
            assert_eq!(slug.get(), "hello-world-how-s-it-going");
        }

        #[test]
        fn empty_result_error() {
            let result = Slug::from_title("!@#$%^&*()");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.reason, DomainErrorKind::Empty));
        }

        #[test]
        fn empty_input() {
            let result = Slug::from_title("");
            assert!(result.is_err());
        }

        #[test]
        fn only_spaces() {
            let result = Slug::from_title("   ");
            assert!(result.is_err());
        }

        #[test]
        fn underscores_converted() {
            let slug = Slug::from_title("snake_case_title").unwrap();
            assert_eq!(slug.get(), "snake-case-title");
        }

        #[test]
        fn tabs_and_newlines() {
            let slug = Slug::from_title("tabs\tand\nnewlines").unwrap();
            assert_eq!(slug.get(), "tabs-and-newlines");
        }

        #[test]
        fn unicode_removed() {
            let slug = Slug::from_title("Café Résumé").unwrap();
            assert_eq!(slug.get(), "caf-r-sum");
        }

        #[test]
        fn emoji_removed() {
            let slug = Slug::from_title("Hello 🎉 World").unwrap();
            assert_eq!(slug.get(), "hello-world");
        }
    }

    // ========================================================================
    // Error Message Tests
    // ========================================================================

    mod error_messages {
        use super::*;

        #[test]
        fn error_includes_format_name() {
            let result = Slug::new("Invalid Slug".to_string());
            let err = result.unwrap_err();
            assert_eq!(err.format_name, "slug");
        }

        #[test]
        fn error_includes_example() {
            let result = Slug::new("Invalid".to_string());
            let err = result.unwrap_err();
            assert_eq!(err.example, "my-first-post");
        }

        #[test]
        fn error_display_format() {
            let result = Slug::new("Hello World".to_string());
            let err = result.unwrap_err();
            let msg = err.to_string();
            assert!(msg.contains("slug"));
            assert!(msg.contains("my-first-post"));
        }

        #[test]
        fn description_returns_expected() {
            assert_eq!(
                ValidSlug::description(),
                "URL-safe slug (lowercase alphanumeric with hyphens)"
            );
        }
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn single_char_slug() {
            assert!(Slug::new("a".to_string()).is_ok());
            assert!(Slug::new("1".to_string()).is_ok());
            assert!(Slug::new("-".to_string()).is_err());
        }

        #[test]
        fn two_char_slug_with_hyphen() {
            // "a-" is invalid (trailing hyphen)
            assert!(Slug::new("a-".to_string()).is_err());
            // "-a" is invalid (leading hyphen)
            assert!(Slug::new("-a".to_string()).is_err());
        }

        #[test]
        fn three_char_slug_with_hyphen() {
            // "a-b" is valid
            assert!(Slug::new("a-b".to_string()).is_ok());
        }

        #[test]
        fn boundary_length_127() {
            let slug = "a".repeat(127);
            assert!(Slug::new(slug).is_ok());
        }

        #[test]
        fn boundary_length_128() {
            let slug = "a".repeat(128);
            assert!(Slug::new(slug).is_ok());
        }

        #[test]
        fn boundary_length_129() {
            let slug = "a".repeat(129);
            assert!(Slug::new(slug).is_err());
        }
    }
}

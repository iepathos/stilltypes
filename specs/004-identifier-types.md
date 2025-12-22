---
number: 4
title: Identifier Types
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-12-22
---

# Specification 004: Identifier Types

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: none

## Philosophy Alignment

This specification follows the [Stillwater Philosophy](../../../stillwater/PHILOSOPHY.md):

- **Parse, Don't Validate** (§7): `Slug` encodes URL-safety at the type level. Once constructed, it's guaranteed to be a valid URL path component.
- **Errors Should Tell Stories** (§3): Errors identify the specific invalid character and position, with examples of valid formats.
- **Composition Over Complexity** (§4): Uses stillwater's string predicates as building blocks.
- **Pragmatism Over Purity** (§6): Provides `from_title()` for practical conversion from prose, not just validation.

## Context

Web applications frequently use human-readable identifiers in URLs, file paths, and database keys. These identifiers (often called "slugs") must be URL-safe, case-insensitive, and readable. Common examples include:

- Blog post URLs: `/posts/my-first-post`
- Product SKUs: `laptop-pro-15`
- User handles: `@john-doe`
- File names: `report-2024-q1`

While basic string validation is possible with stillwater's `is_alphanumeric()` and `all_chars()` predicates, slugs have specific rules that benefit from domain-specific validation:

1. Must start and end with alphanumeric characters
2. Can contain hyphens (but not consecutive)
3. Must be lowercase (for URL consistency)
4. Have reasonable length limits

## Objective

Add an `identifiers` feature to stilltypes providing refined types for common identifier formats, starting with `Slug`. The implementation demonstrates character-level validation with position-aware error messages.

## Requirements

### Functional Requirements

1. **Slug Type**
   - Accept strings matching pattern: `^[a-z0-9]+(-[a-z0-9]+)*$`
   - Rules:
     - Only lowercase letters, digits, and hyphens allowed
     - Must start with a letter or digit
     - Must end with a letter or digit
     - No consecutive hyphens (`--`)
     - Length: 1-128 characters (configurable via const generic optional)
   - Provide normalization: `from_string(s)` that converts to slug format
   - Provide conversion from title: `from_title("My First Post")` -> `"my-first-post"`

2. **Username Type (Optional)**
   - Pattern: `^[a-z][a-z0-9_]*$`
   - Rules:
     - Starts with lowercase letter
     - Contains only lowercase letters, digits, underscores
     - Length: 3-32 characters
   - Common for user handles, accounts

3. **Sku Type (Optional)**
   - Pattern: `^[A-Z0-9]+(-[A-Z0-9]+)*$`
   - Rules:
     - Uppercase letters, digits, hyphens
     - Similar structure to slug but uppercase
   - Common for product identifiers

### Non-Functional Requirements

- Zero external dependencies (pure string operations)
- Serde support when `serde` feature is enabled
- Error messages should identify the specific character/position that's invalid
- All predicates must be zero-sized types (ZSTs)
- Consider Unicode normalization (NFC) for input strings

## Acceptance Criteria

- [ ] `Slug` type validates strings matching `^[a-z0-9]+(-[a-z0-9]+)*$`
- [ ] Validation rejects uppercase, spaces, consecutive hyphens, leading/trailing hyphens
- [ ] Error messages identify invalid character and position
- [ ] `SlugExt` provides `from_title()` for converting prose to slug format
- [ ] Length limits are enforced with appropriate error messages
- [ ] Unit tests cover valid slugs: "hello", "hello-world", "post-123", "a1-b2-c3"
- [ ] Unit tests cover invalid slugs: "Hello", "hello--world", "-hello", "hello-", "hello world"
- [ ] `from_title()` handles edge cases: leading/trailing spaces, multiple spaces, special chars
- [ ] Serde integration tests pass when feature enabled
- [ ] `examples/slug_validation.rs` demonstrates error accumulation pattern
- [ ] README.md feature table updated with identifier types
- [ ] lib.rs feature table updated with identifier types
- [ ] `full` feature includes `identifiers`

## Technical Details

### Implementation Approach

```rust
// src/identifiers.rs

use stillwater::refined::Refined;
use crate::error::{DomainError, DomainErrorKind};

/// Predicate for valid URL slugs.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidSlug;

impl Predicate<String> for ValidSlug {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(DomainError {
                format_name: "slug",
                value: value.clone(),
                reason: DomainErrorKind::Empty,
                example: "my-first-post",
            });
        }

        if value.len() > 128 {
            return Err(DomainError {
                format_name: "slug",
                value: format!("{}...", &value[..32]),
                reason: DomainErrorKind::TooLong { max: 128, actual: value.len() },
                example: "my-first-post",
            });
        }

        let chars: Vec<char> = value.chars().collect();

        // Check first character
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

        // Check last character
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
}

/// A validated URL-safe slug.
pub type Slug = Refined<String, ValidSlug>;

/// Extension trait for slug operations.
pub trait SlugExt {
    /// Convert a title or prose string into a valid slug.
    ///
    /// - Converts to lowercase
    /// - Replaces spaces and underscores with hyphens
    /// - Removes non-alphanumeric characters (except hyphens)
    /// - Collapses consecutive hyphens
    /// - Trims leading/trailing hyphens
    ///
    /// # Example
    /// ```
    /// use stilltypes::identifiers::Slug;
    ///
    /// let slug = Slug::from_title("My First Blog Post!").unwrap();
    /// assert_eq!(slug.as_ref(), "my-first-blog-post");
    /// ```
    fn from_title(title: &str) -> Result<Slug, DomainError>;
}

impl SlugExt for Slug {
    fn from_title(title: &str) -> Result<Slug, DomainError> {
        let slug: String = title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        Slug::new(slug)
    }
}
```

### Feature Flag

```toml
[features]
identifiers = []  # No external dependencies
```

### Error Messages

```
"invalid slug: cannot be empty (example: my-first-post)"
"invalid slug: 'H' at position 0 is not allowed (example: my-first-post)"
"invalid slug: ' ' at position 5 is not allowed (example: my-first-post)"
"invalid slug: no consecutive hyphens allowed (example: my-first-post)"
"invalid slug: too long (150 chars, max 128) (example: my-first-post)"
```

## Error Accumulation Example

Following the "Fail Completely" pattern (PHILOSOPHY.md §2), identifier types integrate with `Validation::all()`:

```rust
use stilltypes::prelude::*;
use stilltypes::identifiers::Slug;
use stillwater::validation::{Validation, ValidateAll};

/// Raw blog post input.
struct BlogPostInput {
    title: String,
    slug: Option<String>,  // Optional custom slug
    tags: Vec<String>,
}

/// Validated blog post with guaranteed-valid identifiers.
struct ValidBlogPost {
    title: String,
    slug: Slug,
    tags: Vec<Slug>,
}

fn validate_blog_post(input: BlogPostInput) -> Validation<ValidBlogPost, Vec<DomainError>> {
    // Use custom slug if provided, otherwise generate from title
    let slug_v = match input.slug {
        Some(custom) => Validation::from_result(Slug::new(custom).map_err(|e| vec![e])),
        None => Validation::from_result(Slug::from_title(&input.title).map_err(|e| vec![e])),
    };

    let tags_v: Validation<Vec<Slug>, Vec<DomainError>> = input.tags
        .into_iter()
        .map(|tag| Validation::from_result(Slug::from_title(&tag).map_err(|e| vec![e])))
        .collect();

    (slug_v, tags_v)
        .validate_all()
        .map(|(slug, tags)| ValidBlogPost {
            title: input.title,
            slug,
            tags,
        })
}
```

## Pure Core Example

Once validated, identifier types enable pure URL generation:

```rust
/// Pure function - generates blog post URL from validated slug.
fn blog_post_url(slug: &Slug) -> String {
    format!("/posts/{}", slug.get())
}

/// Pure function - generates tag archive URL from validated slugs.
fn tag_url(tag: &Slug) -> String {
    format!("/tags/{}", tag.get())
}

/// Pure function - generates sitemap entries (no validation needed).
fn sitemap_entries(posts: &[(String, Slug)]) -> Vec<String> {
    posts.iter()
        .map(|(_, slug)| format!("https://example.com/posts/{}", slug.get()))
        .collect()
}
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**: `src/lib.rs`, `src/prelude.rs`, `Cargo.toml`
- **External Dependencies**: None

## Testing Strategy

- **Unit Tests**:
  - Valid slugs: "a", "hello", "hello-world", "post-123", "a1-b2-c3", "x"*128
  - Invalid slugs:
    - Empty: ""
    - Uppercase: "Hello", "HELLO"
    - Spaces: "hello world"
    - Special chars: "hello@world", "hello.world"
    - Consecutive hyphens: "hello--world", "a---b"
    - Leading hyphen: "-hello"
    - Trailing hyphen: "hello-"
    - Too long: "x"*129

- **from_title() Tests**:
  - "My First Post" -> "my-first-post"
  - "  Spaces  Everywhere  " -> "spaces-everywhere"
  - "Special!@#Characters" -> "special-characters"
  - "Already-a-Slug" -> "already-a-slug"
  - "123 Numbers First" -> "123-numbers-first"
  - "" -> Error (empty result)

- **Integration Tests**: Serde round-trip

## Documentation Requirements

### Code Documentation
- Full rustdoc with examples for each type and trait
- Module-level documentation explaining slug patterns and use cases

### lib.rs Feature Table Update
Add row to the feature table in `src/lib.rs`:
```markdown
//! | `identifiers` | [`Slug`](identifiers::Slug) | - |
```

### README.md Updates
Add to feature table:
```markdown
| `identifiers` | `Slug` | - |
```

Add usage section:
```markdown
### URL Slugs

\`\`\`rust,ignore
use stilltypes::identifiers::{Slug, SlugExt};

// Validate existing slug
let slug = Slug::new("my-first-post".to_string())?;

// Convert from title
let slug = Slug::from_title("My First Blog Post!")?;
assert_eq!(slug.get(), "my-first-blog-post");
\`\`\`
```

### Example File
Create `examples/slug_validation.rs`:
- Demonstrate blog post slug generation with error accumulation
- Show `from_title()` conversion from various inputs
- Include tag slugification
- Pattern after `examples/form_validation.rs`

## Implementation Notes

- Use `is_ascii_lowercase()` rather than `is_lowercase()` to avoid Unicode complications
- The `from_title()` method should handle common title formats gracefully
- Consider whether to allow digits at the start (current spec allows this)
- Max length of 128 is arbitrary but reasonable for URLs
- Consider a `SlugBuilder` for complex transformations

## Migration and Compatibility

- New feature, no breaking changes
- Optional feature flag means no impact on existing users

## Future Extensions

- `Username` type for user handles
- `Sku` type for product codes
- `ApiKey` type for API authentication tokens (with masking like credit cards)
- `FileSlug` type with OS-specific character restrictions

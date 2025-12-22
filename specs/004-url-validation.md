---
number: 4
title: URL Validation
category: foundation
priority: high
status: draft
dependencies: [1, 2]
created: 2025-12-21
---

# Specification 4: URL Validation

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 1 (Project Foundation), Spec 2 (Error Types)

## Context

URLs are fundamental to web applications and need validation for various use cases:
- Any valid URL (RFC 3986)
- HTTP/HTTPS URLs only (for web APIs)
- HTTPS-only URLs (for security-sensitive applications)

Rather than implementing our own URL parser, Platypus uses the well-tested `url` crate which implements the WHATWG URL Standard (a superset of RFC 3986). This follows the "Parse, Don't Validate" principle.

The key innovation here is demonstrating stillwater's predicate composition. Instead of creating monolithic validators, we create simple predicates (`ValidUrl`, `HttpScheme`, `HttpsOnly`) and compose them using stillwater's `And` combinator.

## Objective

Implement URL validation predicates and type aliases:
- `ValidUrl` - any RFC 3986 compliant URL
- `HttpScheme` - scheme must be http or https
- `HttpsOnly` - scheme must be https
- Type aliases using stillwater's `And` combinator for composition

## Requirements

### Functional Requirements

1. **ValidUrl Predicate**
   - Implements `stillwater::refined::Predicate<String>`
   - Uses `url::Url::parse()` for validation
   - Returns `DomainError` with appropriate context on failure
   - Provides `description()` returning "RFC 3986 URL"

2. **HttpScheme Predicate**
   - Validates that scheme is "http" or "https"
   - Returns `DomainError` with scheme-specific message
   - Provides `description()` returning "HTTP or HTTPS scheme"

3. **HttpsOnly Predicate**
   - Validates that scheme is "https"
   - Returns `DomainError` with scheme-specific message
   - Provides `description()` returning "HTTPS scheme only"

4. **Type Aliases**
   - `Url = Refined<String, ValidUrl>` - any valid URL
   - `HttpUrl = Refined<String, And<ValidUrl, HttpScheme>>` - http or https
   - `SecureUrl = Refined<String, And<ValidUrl, HttpsOnly>>` - https only

5. **Feature Gating**
   - Only compiled when `url` feature is enabled

### Non-Functional Requirements

1. **RFC Compliance**: Follows WHATWG URL Standard (RFC 3986 superset)
2. **Composition**: Demonstrates stillwater's `And` combinator
3. **Zero Unsafe Code**: Pure safe Rust implementation

## Acceptance Criteria

- [ ] `ValidUrl` struct implements `Predicate<String>`
- [ ] `HttpScheme` struct implements `Predicate<String>`
- [ ] `HttpsOnly` struct implements `Predicate<String>`
- [ ] `Url` type alias defined and exported
- [ ] `HttpUrl` type alias uses `And<ValidUrl, HttpScheme>`
- [ ] `SecureUrl` type alias uses `And<ValidUrl, HttpsOnly>`
- [ ] Invalid URLs return `DomainErrorKind::InvalidFormat`
- [ ] Wrong scheme returns `DomainErrorKind::InvalidComponent`
- [ ] Compiles only with `url` feature enabled
- [ ] Unit tests for all predicates
- [ ] Composition with `And` works correctly

## Technical Details

### Implementation Approach

```rust
// src/url.rs

use crate::error::{DomainError, DomainErrorKind};
use stillwater::refined::{And, Predicate, Refined};
use url::Url as UrlParser;

/// Any valid RFC 3986 URL.
///
/// Uses the `url` crate for parsing, which implements the WHATWG URL Standard.
///
/// # Example
/// ```
/// use platypus::url::Url;
///
/// let url = Url::new("https://example.com/path".to_string());
/// assert!(url.is_ok());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidUrl;

impl Predicate<String> for ValidUrl {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        UrlParser::parse(value).map(|_| ()).map_err(|_| DomainError {
            format_name: "URL",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "scheme://host/path",
            },
            example: "https://example.com",
        })
    }

    fn description() -> &'static str {
        "RFC 3986 URL"
    }
}

/// URL scheme must be http or https.
///
/// This predicate validates only the scheme, not the overall URL validity.
/// For a complete HTTP URL, use `HttpUrl` which combines `ValidUrl` and `HttpScheme`.
///
/// # Example
/// ```
/// use platypus::url::HttpUrl;
///
/// let http = HttpUrl::new("http://example.com".to_string());
/// assert!(http.is_ok());
///
/// let ftp = HttpUrl::new("ftp://example.com".to_string());
/// assert!(ftp.is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct HttpScheme;

impl Predicate<String> for HttpScheme {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        let parsed = UrlParser::parse(value).map_err(|_| DomainError {
            format_name: "HTTP URL",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "valid URL",
            },
            example: "https://example.com",
        })?;

        match parsed.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(DomainError {
                format_name: "HTTP URL",
                value: value.clone(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "scheme",
                    reason: format!("expected http or https, got {}", scheme),
                },
                example: "https://example.com",
            }),
        }
    }

    fn description() -> &'static str {
        "HTTP or HTTPS scheme"
    }
}

/// URL scheme must be https (secure).
///
/// This predicate validates only the scheme, not the overall URL validity.
/// For a complete secure URL, use `SecureUrl` which combines `ValidUrl` and `HttpsOnly`.
///
/// # Example
/// ```
/// use platypus::url::SecureUrl;
///
/// let https = SecureUrl::new("https://example.com".to_string());
/// assert!(https.is_ok());
///
/// let http = SecureUrl::new("http://example.com".to_string());
/// assert!(http.is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct HttpsOnly;

impl Predicate<String> for HttpsOnly {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        let parsed = UrlParser::parse(value).map_err(|_| DomainError {
            format_name: "HTTPS URL",
            value: value.clone(),
            reason: DomainErrorKind::InvalidFormat {
                expected: "valid URL",
            },
            example: "https://example.com",
        })?;

        if parsed.scheme() == "https" {
            Ok(())
        } else {
            Err(DomainError {
                format_name: "HTTPS URL",
                value: value.clone(),
                reason: DomainErrorKind::InvalidComponent {
                    component: "scheme",
                    reason: format!("expected https, got {}", parsed.scheme()),
                },
                example: "https://example.com",
            })
        }
    }

    fn description() -> &'static str {
        "HTTPS scheme only"
    }
}

/// Any valid URL (RFC 3986).
pub type Url = Refined<String, ValidUrl>;

/// HTTP or HTTPS URL.
///
/// Composed using stillwater's `And` combinator.
pub type HttpUrl = Refined<String, And<ValidUrl, HttpScheme>>;

/// HTTPS-only URL (secure).
///
/// Composed using stillwater's `And` combinator.
pub type SecureUrl = Refined<String, And<ValidUrl, HttpsOnly>>;
```

### Module Integration

```rust
// In src/lib.rs
#[cfg(feature = "url")]
pub mod url;

#[cfg(feature = "url")]
pub use url::{HttpScheme, HttpUrl, HttpsOnly, SecureUrl, Url, ValidUrl};
```

## Dependencies

- **Prerequisites**: Spec 1 (crate structure), Spec 2 (error types)
- **Affected Components**: prelude.rs, lib.rs exports
- **External Dependencies**: `url` crate (version 2)

## Testing Strategy

### ValidUrl Tests

```rust
#[test]
fn valid_https_url() {
    assert!(Url::new("https://example.com".to_string()).is_ok());
}

#[test]
fn valid_http_url() {
    assert!(Url::new("http://example.com".to_string()).is_ok());
}

#[test]
fn valid_with_path() {
    assert!(Url::new("https://example.com/path/to/resource".to_string()).is_ok());
}

#[test]
fn valid_with_query() {
    assert!(Url::new("https://example.com?foo=bar&baz=qux".to_string()).is_ok());
}

#[test]
fn valid_with_fragment() {
    assert!(Url::new("https://example.com#section".to_string()).is_ok());
}

#[test]
fn valid_with_port() {
    assert!(Url::new("https://example.com:8080".to_string()).is_ok());
}

#[test]
fn valid_ftp_url() {
    // ValidUrl accepts any scheme
    assert!(Url::new("ftp://files.example.com".to_string()).is_ok());
}

#[test]
fn invalid_missing_scheme() {
    assert!(Url::new("example.com".to_string()).is_err());
}

#[test]
fn invalid_malformed() {
    assert!(Url::new("not a url at all".to_string()).is_err());
}
```

### HttpUrl Tests

```rust
#[test]
fn http_url_accepts_http() {
    assert!(HttpUrl::new("http://example.com".to_string()).is_ok());
}

#[test]
fn http_url_accepts_https() {
    assert!(HttpUrl::new("https://example.com".to_string()).is_ok());
}

#[test]
fn http_url_rejects_ftp() {
    let result = HttpUrl::new("ftp://example.com".to_string());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.reason, DomainErrorKind::InvalidComponent { .. }));
}

#[test]
fn http_url_rejects_file() {
    assert!(HttpUrl::new("file:///path/to/file".to_string()).is_err());
}
```

### SecureUrl Tests

```rust
#[test]
fn secure_url_accepts_https() {
    assert!(SecureUrl::new("https://example.com".to_string()).is_ok());
}

#[test]
fn secure_url_rejects_http() {
    let result = SecureUrl::new("http://example.com".to_string());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.reason, DomainErrorKind::InvalidComponent { .. }));
}
```

### Composition Tests

```rust
#[test]
fn and_combinator_validates_both_predicates() {
    // This tests that stillwater's And combinator works correctly
    // The HttpUrl type is And<ValidUrl, HttpScheme>

    // Invalid URL should fail ValidUrl
    assert!(HttpUrl::new("not a url".to_string()).is_err());

    // Valid FTP should fail HttpScheme
    assert!(HttpUrl::new("ftp://example.com".to_string()).is_err());

    // Valid HTTPS should pass both
    assert!(HttpUrl::new("https://example.com".to_string()).is_ok());
}
```

## Documentation Requirements

- **Code Documentation**: Doc comments with examples on all predicates and type aliases
- **Composition Example**: Show how And combinator works
- **User Documentation**: Usage examples in README

## Implementation Notes

- The `url` crate implements WHATWG URL Standard which is stricter than RFC 3986 in some ways
- `HttpScheme` and `HttpsOnly` parse the URL again - this is intentional for modularity
- The double parsing is minimal overhead and keeps predicates composable
- Consider adding `Url::host()`, `Url::path()` helper methods in future

## Migration and Compatibility

N/A - New type with no existing code to migrate.

## Future Considerations

Not in scope for initial implementation:
- `Url::to_url()` - convert to `url::Url`
- `Url::host()` - extract host
- `Url::path()` - extract path
- `Url::with_path()` - builder pattern
- `LocalhostUrl` - for development URLs

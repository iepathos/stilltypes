//! URL validation types.
//!
//! Provides RFC 3986 compliant URL validation using the `url` crate (WHATWG URL Standard).
//!
//! This module demonstrates stillwater's predicate composition using the `And` combinator:
//! - `Url` - any valid RFC 3986 URL
//! - `HttpUrl` - composed as `And<ValidUrl, HttpScheme>`
//! - `SecureUrl` - composed as `And<ValidUrl, HttpsOnly>`
//!
//! # Example
//!
//! ```
//! use platypus::url::{Url, HttpUrl, SecureUrl};
//!
//! // Any valid URL
//! let url = Url::new("https://example.com".to_string());
//! assert!(url.is_ok());
//!
//! // HTTP or HTTPS only
//! let http = HttpUrl::new("http://example.com".to_string());
//! assert!(http.is_ok());
//!
//! // HTTPS only (secure)
//! let https = SecureUrl::new("https://example.com".to_string());
//! assert!(https.is_ok());
//!
//! let insecure = SecureUrl::new("http://example.com".to_string());
//! assert!(insecure.is_err());
//! ```

use crate::error::{DomainError, DomainErrorKind};
use stillwater::refined::{And, Predicate, Refined};
use url::Url as UrlParser;

/// Any valid RFC 3986 URL.
///
/// Uses the `url` crate for parsing, which implements the WHATWG URL Standard.
///
/// # Example
///
/// ```
/// use platypus::url::Url;
///
/// let url = Url::new("https://example.com/path".to_string());
/// assert!(url.is_ok());
///
/// let invalid = Url::new("not a url".to_string());
/// assert!(invalid.is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidUrl;

impl Predicate<String> for ValidUrl {
    type Error = DomainError;

    fn check(value: &String) -> Result<(), Self::Error> {
        UrlParser::parse(value)
            .map(|_| ())
            .map_err(|_| DomainError {
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
///
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
///
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
///
/// A `String` that has been validated to be a properly formatted URL
/// according to RFC 3986 (via the WHATWG URL Standard).
///
/// # Example
///
/// ```
/// use platypus::url::Url;
///
/// let url = Url::new("https://example.com/path?query=value".to_string()).unwrap();
/// assert_eq!(url.get(), "https://example.com/path?query=value");
/// ```
pub type Url = Refined<String, ValidUrl>;

/// HTTP or HTTPS URL.
///
/// Composed using stillwater's `And` combinator to validate both
/// URL structure and scheme.
///
/// # Example
///
/// ```
/// use platypus::url::HttpUrl;
///
/// let http = HttpUrl::new("http://example.com".to_string()).unwrap();
/// let https = HttpUrl::new("https://example.com".to_string()).unwrap();
///
/// // FTP is rejected
/// let ftp = HttpUrl::new("ftp://files.example.com".to_string());
/// assert!(ftp.is_err());
/// ```
pub type HttpUrl = Refined<String, And<ValidUrl, HttpScheme>>;

/// HTTPS-only URL (secure).
///
/// Composed using stillwater's `And` combinator to validate both
/// URL structure and secure scheme.
///
/// # Example
///
/// ```
/// use platypus::url::SecureUrl;
///
/// let secure = SecureUrl::new("https://api.example.com".to_string()).unwrap();
///
/// // HTTP is rejected
/// let insecure = SecureUrl::new("http://example.com".to_string());
/// assert!(insecure.is_err());
/// ```
pub type SecureUrl = Refined<String, And<ValidUrl, HttpsOnly>>;

#[cfg(test)]
mod tests {
    use super::*;

    // ValidUrl tests
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

    #[test]
    fn valid_url_description() {
        assert_eq!(ValidUrl::description(), "RFC 3986 URL");
    }

    // HttpUrl tests
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
        // HttpUrl uses And combinator which returns AndError
        // FTP passes ValidUrl but fails HttpScheme, so it's AndError::Second
        let err = result.unwrap_err();
        match err {
            stillwater::refined::AndError::Second(domain_err) => {
                assert!(matches!(
                    domain_err.reason,
                    DomainErrorKind::InvalidComponent { .. }
                ));
            }
            _ => panic!("Expected AndError::Second for scheme rejection"),
        }
    }

    #[test]
    fn http_url_rejects_file() {
        assert!(HttpUrl::new("file:///path/to/file".to_string()).is_err());
    }

    #[test]
    fn http_scheme_description() {
        assert_eq!(HttpScheme::description(), "HTTP or HTTPS scheme");
    }

    // SecureUrl tests
    #[test]
    fn secure_url_accepts_https() {
        assert!(SecureUrl::new("https://example.com".to_string()).is_ok());
    }

    #[test]
    fn secure_url_rejects_http() {
        let result = SecureUrl::new("http://example.com".to_string());
        assert!(result.is_err());
        // SecureUrl uses And combinator which returns AndError
        // HTTP passes ValidUrl but fails HttpsOnly, so it's AndError::Second
        let err = result.unwrap_err();
        match err {
            stillwater::refined::AndError::Second(domain_err) => {
                assert!(matches!(
                    domain_err.reason,
                    DomainErrorKind::InvalidComponent { .. }
                ));
            }
            _ => panic!("Expected AndError::Second for scheme rejection"),
        }
    }

    #[test]
    fn https_only_description() {
        assert_eq!(HttpsOnly::description(), "HTTPS scheme only");
    }

    // Composition tests
    #[test]
    fn and_combinator_validates_both_predicates() {
        // Invalid URL should fail ValidUrl
        assert!(HttpUrl::new("not a url".to_string()).is_err());

        // Valid FTP should fail HttpScheme
        assert!(HttpUrl::new("ftp://example.com".to_string()).is_err());

        // Valid HTTPS should pass both
        assert!(HttpUrl::new("https://example.com".to_string()).is_ok());
    }

    // Error message tests
    #[test]
    fn invalid_url_error_includes_format_name() {
        let result = Url::new("invalid".to_string());
        let err = result.unwrap_err();
        assert_eq!(err.format_name, "URL");
    }

    #[test]
    fn invalid_url_error_includes_example() {
        let result = Url::new("invalid".to_string());
        let err = result.unwrap_err();
        assert_eq!(err.example, "https://example.com");
    }

    #[test]
    fn scheme_error_is_invalid_component() {
        let result = SecureUrl::new("http://example.com".to_string());
        let err = result.unwrap_err();
        // SecureUrl uses And combinator, extract the underlying DomainError
        match err {
            stillwater::refined::AndError::Second(domain_err) => match domain_err.reason {
                DomainErrorKind::InvalidComponent { component, reason } => {
                    assert_eq!(component, "scheme");
                    assert!(reason.contains("https"));
                }
                _ => panic!("Expected InvalidComponent error"),
            },
            _ => panic!("Expected AndError::Second"),
        }
    }
}

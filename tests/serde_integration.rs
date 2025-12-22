//! Serde integration tests for stilltypes.
//!
//! Tests validation-on-deserialize behavior using stillwater's built-in
//! serde implementation for `Refined<T, P>`.

#[cfg(all(feature = "serde", feature = "email"))]
mod email_serde {
    use stilltypes::email::Email;

    #[test]
    fn email_deserializes() {
        let json = r#""user@example.com""#;
        let email: Email = serde_json::from_str(json).unwrap();
        assert_eq!(email.get(), "user@example.com");
    }

    #[test]
    fn invalid_email_fails() {
        let json = r#""not-an-email""#;
        let result: Result<Email, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn email_serializes() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let json = serde_json::to_string(&email).unwrap();
        assert_eq!(json, r#""test@example.com""#);
    }

    #[test]
    fn email_roundtrip() {
        let original = Email::new("roundtrip@example.com".to_string()).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Email = serde_json::from_str(&json).unwrap();
        assert_eq!(original.get(), restored.get());
    }
}

#[cfg(all(feature = "serde", feature = "url"))]
mod url_serde {
    use stilltypes::url::{HttpUrl, SecureUrl, Url};

    #[test]
    fn url_deserializes() {
        let json = r#""https://example.com""#;
        let url: Url = serde_json::from_str(json).unwrap();
        // URL normalization may add trailing slash
        assert!(url.get().starts_with("https://example.com"));
    }

    #[test]
    fn http_url_deserializes() {
        let json = r#""http://example.com""#;
        let url: HttpUrl = serde_json::from_str(json).unwrap();
        assert!(url.get().starts_with("http://"));
    }

    #[test]
    fn http_url_rejects_ftp() {
        let json = r#""ftp://example.com""#;
        let result: Result<HttpUrl, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn secure_url_deserializes() {
        let json = r#""https://example.com""#;
        let url: SecureUrl = serde_json::from_str(json).unwrap();
        assert!(url.get().starts_with("https://"));
    }

    #[test]
    fn secure_url_rejects_http() {
        let json = r#""http://example.com""#;
        let result: Result<SecureUrl, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn url_roundtrip() {
        let original = Url::new("https://example.com/path".to_string()).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Url = serde_json::from_str(&json).unwrap();
        assert_eq!(original.get(), restored.get());
    }
}

#[cfg(all(feature = "serde", feature = "uuid"))]
mod uuid_serde {
    use stilltypes::uuid::{Uuid, UuidV4, UuidV7};

    #[test]
    fn uuid_deserializes() {
        let json = r#""550e8400-e29b-41d4-a716-446655440000""#;
        let uuid: Uuid = serde_json::from_str(json).unwrap();
        assert!(uuid.get().contains("550e8400"));
    }

    #[test]
    fn uuid_v4_deserializes() {
        let json = r#""550e8400-e29b-41d4-a716-446655440000""#;
        let uuid: UuidV4 = serde_json::from_str(json).unwrap();
        assert!(uuid.get().contains("550e8400"));
    }

    #[test]
    fn uuid_v4_rejects_v7() {
        // v7 UUID fails for UuidV4 type
        let json = r#""018f6b8e-e4a0-7000-8000-000000000000""#;
        let result: Result<UuidV4, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn uuid_v7_deserializes() {
        let json = r#""018f6b8e-e4a0-7000-8000-000000000000""#;
        let uuid: UuidV7 = serde_json::from_str(json).unwrap();
        assert!(uuid.get().contains("018f6b8e"));
    }

    #[test]
    fn uuid_roundtrip() {
        let original = Uuid::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Uuid = serde_json::from_str(&json).unwrap();
        assert_eq!(original.get(), restored.get());
    }
}

#[cfg(all(feature = "serde", feature = "phone"))]
mod phone_serde {
    use stilltypes::phone::{PhoneNumber, PhoneNumberExt};

    #[test]
    fn phone_deserializes() {
        let json = r#""+14155551234""#;
        let phone: PhoneNumber = serde_json::from_str(json).unwrap();
        assert_eq!(phone.to_e164(), "+14155551234");
    }

    #[test]
    fn phone_with_formatting_deserializes() {
        let json = r#""+1 (415) 555-1234""#;
        let phone: PhoneNumber = serde_json::from_str(json).unwrap();
        assert_eq!(phone.to_e164(), "+14155551234");
    }

    #[test]
    fn invalid_phone_fails() {
        let json = r#""not-a-phone""#;
        let result: Result<PhoneNumber, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn phone_roundtrip() {
        let original = PhoneNumber::new("+14155551234".to_string()).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: PhoneNumber = serde_json::from_str(&json).unwrap();
        assert_eq!(original.to_e164(), restored.to_e164());
    }
}

#[cfg(all(feature = "serde", feature = "financial"))]
mod financial_serde {
    use stilltypes::financial::{CreditCardExt, CreditCardNumber, Iban, IbanExt};

    #[test]
    fn iban_deserializes() {
        let json = r#""DE89370400440532013000""#;
        let iban: Iban = serde_json::from_str(json).unwrap();
        assert_eq!(iban.country_code(), "DE");
    }

    #[test]
    fn iban_lowercase_deserializes() {
        let json = r#""de89370400440532013000""#;
        let iban: Iban = serde_json::from_str(json).unwrap();
        assert_eq!(iban.country_code(), "de");
    }

    #[test]
    fn invalid_iban_fails() {
        let json = r#""DE00370400440532013000""#;
        let result: Result<Iban, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn credit_card_deserializes() {
        let json = r#""4111111111111111""#;
        let card: CreditCardNumber = serde_json::from_str(json).unwrap();
        assert_eq!(card.last_four(), "1111");
    }

    #[test]
    fn credit_card_with_spaces_deserializes() {
        let json = r#""4111 1111 1111 1111""#;
        let card: CreditCardNumber = serde_json::from_str(json).unwrap();
        assert_eq!(card.last_four(), "1111");
    }

    #[test]
    fn invalid_luhn_fails() {
        let json = r#""4111111111111112""#;
        let result: Result<CreditCardNumber, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn iban_roundtrip() {
        let original = Iban::new("DE89370400440532013000".to_string()).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Iban = serde_json::from_str(&json).unwrap();
        assert_eq!(original.get(), restored.get());
    }

    #[test]
    fn credit_card_roundtrip() {
        let original = CreditCardNumber::new("4111111111111111".to_string()).unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: CreditCardNumber = serde_json::from_str(&json).unwrap();
        assert_eq!(original.get(), restored.get());
    }
}

#[cfg(all(feature = "serde", feature = "email", feature = "url"))]
mod composite_serde {
    use serde::{Deserialize, Serialize};
    use stilltypes::prelude::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        email: Email,
        website: Option<Url>,
    }

    #[test]
    fn struct_with_validated_fields_deserializes() {
        let json = r#"{"email": "user@example.com", "website": "https://example.com"}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.email.get(), "user@example.com");
        assert!(user.website.is_some());
    }

    #[test]
    fn invalid_email_fails_struct_deserialization() {
        let json = r#"{"email": "invalid", "website": "https://example.com"}"#;
        let result: Result<User, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn missing_optional_field_ok() {
        let json = r#"{"email": "user@example.com"}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert!(user.website.is_none());
    }

    #[test]
    fn null_optional_field_ok() {
        let json = r#"{"email": "user@example.com", "website": null}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert!(user.website.is_none());
    }

    #[test]
    fn struct_roundtrip_preserves_values() {
        let original = User {
            email: Email::new("test@example.com".to_string()).unwrap(),
            website: Some(Url::new("https://example.com".to_string()).unwrap()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: User = serde_json::from_str(&json).unwrap();
        assert_eq!(original.email.get(), restored.email.get());
        assert_eq!(
            original.website.as_ref().map(|u| u.get()),
            restored.website.as_ref().map(|u| u.get())
        );
    }

    #[test]
    fn invalid_optional_url_fails() {
        let json = r#"{"email": "user@example.com", "website": "not-a-url"}"#;
        let result: Result<User, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}

#[cfg(all(feature = "serde", feature = "full"))]
mod full_integration_serde {
    use serde::{Deserialize, Serialize};
    use stilltypes::prelude::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct RegistrationForm {
        email: Email,
        phone: PhoneNumber,
        website: Option<HttpUrl>,
        #[serde(default)]
        terms_accepted: bool,
    }

    #[test]
    fn complex_form_deserializes() {
        let json = r#"{
            "email": "user@example.com",
            "phone": "+14155551234",
            "website": "https://example.com"
        }"#;
        let form: RegistrationForm = serde_json::from_str(json).unwrap();
        assert_eq!(form.email.get(), "user@example.com");
        assert_eq!(form.phone.to_e164(), "+14155551234");
        assert!(form.website.is_some());
        assert!(!form.terms_accepted);
    }

    #[test]
    fn partial_invalid_fails() {
        let json = r#"{
            "email": "invalid",
            "phone": "+14155551234"
        }"#;
        let result: Result<RegistrationForm, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn default_fields_work() {
        let json = r#"{
            "email": "user@example.com",
            "phone": "+14155551234"
        }"#;
        let form: RegistrationForm = serde_json::from_str(json).unwrap();
        assert!(!form.terms_accepted);
        assert!(form.website.is_none());
    }
}

---
number: 1
title: Project Foundation
category: foundation
priority: critical
status: draft
dependencies: []
created: 2025-12-21
---

# Specification 1: Project Foundation

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: None (this is the foundational spec)

## Context

Platypus is a new crate that extends the Stillwater ecosystem with production-ready domain predicates. Before implementing any domain types, we need a solid project foundation with proper crate structure, dependency management, CI configuration, and feature flags.

The project follows the Stillwater philosophy:
- **Pragmatism Over Purity**: No unnecessary abstractions; just predicates
- **Parse, Don't Validate**: Domain types encode invariants in the type
- **Composition Over Complexity**: Uses stillwater's existing `And`, `Or`, `Not`
- **Errors Should Tell Stories**: Rich error context for helpful messages

## Objective

Establish the complete crate infrastructure including Cargo.toml with feature flags, directory structure, CI pipeline, and initial module stubs that compile and pass all quality checks.

## Requirements

### Functional Requirements

1. **Cargo.toml Configuration**
   - Package metadata (name, version, edition, rust-version, description, license, repository, keywords, categories)
   - Stillwater dependency configured (path-based for development)
   - Optional dependencies for each domain (regex, url, uuid, phonenumber, email_address, iban_validate, creditcard)
   - Feature flags: `default`, `full`, `email`, `url`, `uuid`, `phone`, `financial`, `serde`

2. **Directory Structure**
   ```
   platypus/
   ├── Cargo.toml
   ├── README.md
   ├── LICENSE-MIT
   ├── LICENSE-APACHE
   ├── src/
   │   ├── lib.rs
   │   ├── error.rs      # DomainError type (stub)
   │   └── prelude.rs    # Convenient imports
   ├── tests/
   │   └── integration.rs
   └── examples/
       └── form_validation.rs
   ```

3. **lib.rs Module Structure**
   - Conditional module compilation based on feature flags
   - Public exports for all domain types
   - Prelude module for convenient imports

4. **CI Configuration**
   - Clippy with deny warnings
   - rustfmt check
   - Test suite execution
   - Documentation build

### Non-Functional Requirements

1. **Rust Version**: Edition 2024, rust-version 1.89
2. **Dual License**: MIT OR Apache-2.0
3. **Documentation**: All public items documented
4. **Code Quality**: Zero clippy warnings, formatted code

## Acceptance Criteria

- [ ] Cargo.toml exists with all package metadata fields populated
- [ ] All feature flags defined and compile correctly
- [ ] Stillwater dependency configured and accessible
- [ ] Directory structure matches specification
- [ ] lib.rs compiles with conditional module compilation
- [ ] `cargo build` succeeds with default features
- [ ] `cargo build --all-features` succeeds
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo fmt --check` passes
- [ ] `cargo test` passes (even if no tests yet)
- [ ] `cargo doc` builds without warnings
- [ ] LICENSE-MIT and LICENSE-APACHE files exist
- [ ] Basic README.md exists

## Technical Details

### Implementation Approach

1. Create Cargo.toml with complete package metadata
2. Set up directory structure with placeholder files
3. Implement lib.rs with feature-gated module stubs
4. Create prelude.rs with re-exports
5. Add placeholder error.rs module
6. Create empty integration test file
7. Create placeholder example file
8. Add license files
9. Verify all quality checks pass

### Cargo.toml Structure

```toml
[package]
name = "platypus"
version = "0.1.0"
edition = "2024"
rust-version = "1.89"
description = "Domain-specific refined types for the Stillwater ecosystem"
license = "MIT OR Apache-2.0"
repository = "https://github.com/iepathos/platypus"
keywords = ["validation", "types", "refinement", "domain", "stillwater"]
categories = ["development-tools", "rust-patterns"]

[dependencies]
stillwater = { version = "1.0", path = "../stillwater" }

# Optional - each domain has its own dependency
regex = { version = "1", optional = true }
url = { version = "2", optional = true }
uuid = { version = "1", optional = true }
phonenumber = { version = "0.3", optional = true }
email_address = { version = "0.2", optional = true }
iban_validate = { version = "4", optional = true }
creditcard = { version = "0.3", optional = true }

[features]
default = ["email", "url"]
full = ["email", "url", "uuid", "phone", "financial"]

email = ["dep:email_address"]
url = ["dep:url"]
uuid = ["dep:uuid"]
phone = ["dep:phonenumber"]
financial = ["dep:iban_validate", "dep:creditcard"]

serde = ["stillwater/serde", "uuid?/serde", "url?/serde"]

[dev-dependencies]
serde_json = "1"
```

### lib.rs Module Structure

```rust
//! # Platypus
//!
//! Domain-specific refined types for the Stillwater ecosystem.
//!
//! Platypus provides production-ready predicates and type aliases for common
//! domain types like email addresses, URLs, phone numbers, and more.

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod error;
pub mod prelude;

#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "url")]
pub mod url;

#[cfg(feature = "uuid")]
pub mod uuid;

#[cfg(feature = "phone")]
pub mod phone;

#[cfg(feature = "financial")]
pub mod financial;

pub use error::{DomainError, DomainErrorKind};
```

## Dependencies

- **Prerequisites**: stillwater crate must be accessible (path dependency)
- **Affected Components**: None (greenfield project)
- **External Dependencies**: None for foundation (domain deps added later)

## Testing Strategy

- **Unit Tests**: Not applicable for foundation
- **Integration Tests**: Placeholder test file that compiles
- **Build Tests**: Verify all feature flag combinations compile
- **Quality Tests**: CI checks (clippy, fmt, doc)

## Documentation Requirements

- **Code Documentation**: Module-level docs in lib.rs
- **User Documentation**: Basic README.md with project overview
- **Architecture Updates**: N/A (new project)

## Implementation Notes

- Use `edition = "2024"` which requires Rust 1.89+
- The `#![doc = include_str!("../README.md")]` directive includes README in crate docs
- Feature flags use `dep:` prefix for cleaner optional dependency syntax
- Conditional `uuid?/serde` syntax enables serde for uuid only when both features enabled

## Migration and Compatibility

N/A - This is a new project with no existing code to migrate.

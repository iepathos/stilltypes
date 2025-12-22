//! Network validation example demonstrating error accumulation with stilltypes.
//!
//! This example shows how to validate server configurations with IP addresses,
//! domain names, and port numbers using stillwater's `Validation` for error
//! accumulation.
//!
//! Run with: cargo run --example network_validation --features full

use stilltypes::prelude::*;
use stillwater::validation::{ValidateAll, Validation};

/// Raw server configuration input before validation.
#[derive(Debug)]
struct ServerConfigInput {
    host: String,
    port: u16,
    allowed_ips: Vec<String>,
}

/// Validated server configuration - types guarantee validity.
#[derive(Debug)]
struct ValidServerConfig {
    host: DomainName,
    port: Port,
    allowed_ips: Vec<Ipv4Addr>,
}

/// Validates a server configuration, accumulating all errors.
fn validate_config(input: ServerConfigInput) -> Validation<ValidServerConfig, Vec<DomainError>> {
    let host_v: Validation<DomainName, Vec<DomainError>> =
        Validation::from_result(DomainName::new(input.host).map_err(|e| vec![e]));

    let port_v: Validation<Port, Vec<DomainError>> =
        Validation::from_result(Port::new(input.port).map_err(|e| vec![e]));

    // Validate all IPs and collect errors manually
    let mut valid_ips = Vec::new();
    let mut ip_errors = Vec::new();
    for ip_str in input.allowed_ips {
        match Ipv4Addr::new(ip_str) {
            Ok(ip) => valid_ips.push(ip),
            Err(e) => ip_errors.push(e),
        }
    }
    let ips_v: Validation<Vec<Ipv4Addr>, Vec<DomainError>> = if ip_errors.is_empty() {
        Validation::Success(valid_ips)
    } else {
        Validation::Failure(ip_errors)
    };

    (host_v, port_v, ips_v)
        .validate_all()
        .map(|(host, port, allowed_ips)| ValidServerConfig {
            host,
            port,
            allowed_ips,
        })
}

/// Pure function example - operates on guaranteed-valid data.
fn format_bind_address(ip: &Ipv4Addr, port: &Port) -> String {
    format!("{}:{}", ip.get(), port.get())
}

/// Pure function - no validation needed, types guarantee correctness.
fn is_internal_service(host: &DomainName, port: &Port) -> bool {
    host.get().ends_with(".internal") && port.is_privileged()
}

fn main() {
    println!("Stilltypes Network Validation Example");
    println!("=====================================\n");

    // Example 1: Valid server configuration
    println!("=== Valid Server Configuration ===");
    let valid_config = ServerConfigInput {
        host: "api.example.com".into(),
        port: 8080,
        allowed_ips: vec!["192.168.1.1".into(), "10.0.0.1".into(), "172.16.0.1".into()],
    };

    match validate_config(valid_config) {
        Validation::Success(config) => {
            println!("Configuration valid!");
            println!(
                "  Host: {} (TLD: {:?})",
                config.host.get(),
                config.host.tld()
            );
            println!(
                "  Port: {} (privileged: {}, registered: {})",
                config.port.get(),
                config.port.is_privileged(),
                config.port.is_registered()
            );
            println!("  Allowed IPs:");
            for ip in &config.allowed_ips {
                println!(
                    "    - {} (private: {}, loopback: {})",
                    ip.get(),
                    ip.is_private(),
                    ip.is_loopback()
                );
            }
        }
        Validation::Failure(errors) => {
            println!("Configuration invalid!");
            for err in errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 2: Invalid configuration - multiple errors
    println!("\n=== Invalid Server Configuration ===");
    let invalid_config = ServerConfigInput {
        host: "-bad-hostname".into(),
        port: 0,
        allowed_ips: vec![
            "256.0.0.0".into(),
            "192.168.1.1".into(), // This one is valid
            "not.an.ip".into(),
        ],
    };

    match validate_config(invalid_config) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Configuration failed with {} errors:", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 3: IPv4 address validation
    println!("\n=== IPv4 Address Validation ===");
    let test_ips = [
        "192.168.1.1",
        "10.0.0.1",
        "127.0.0.1",
        "8.8.8.8",
        "169.254.1.1",
        "255.255.255.255",
        "256.0.0.0",
        "1.2.3",
    ];

    for ip_str in test_ips {
        match Ipv4Addr::new(ip_str.to_string()) {
            Ok(ip) => {
                println!("  '{}' -> valid", ip_str);
                println!(
                    "    private: {}, loopback: {}, link-local: {}",
                    ip.is_private(),
                    ip.is_loopback(),
                    ip.is_link_local()
                );
            }
            Err(e) => println!("  '{}' -> {}", ip_str, e),
        }
    }

    // Example 4: IPv6 address validation
    println!("\n=== IPv6 Address Validation ===");
    let test_ipv6 = [
        "::1",
        "::",
        "2001:db8::1",
        "fe80::1",
        "::ffff:192.168.1.1",
        "invalid",
        ":::",
    ];

    for ip_str in test_ipv6 {
        match Ipv6Addr::new(ip_str.to_string()) {
            Ok(ip) => {
                println!("  '{}' -> valid", ip_str);
                println!(
                    "    loopback: {}, unspecified: {}, ipv4-mapped: {}",
                    ip.is_loopback(),
                    ip.is_unspecified(),
                    ip.is_ipv4_mapped()
                );
            }
            Err(e) => println!("  '{}' -> {}", ip_str, e),
        }
    }

    // Example 5: Domain name validation
    println!("\n=== Domain Name Validation ===");
    let test_domains = [
        "example.com",
        "sub.example.com",
        "api.internal",
        "localhost",
        "-invalid.com",
        "also-.invalid.com",
        "exam ple.com",
    ];

    for domain_str in test_domains {
        match DomainName::new(domain_str.to_string()) {
            Ok(domain) => {
                println!("  '{}' -> valid", domain_str);
                println!("    labels: {:?}, tld: {:?}", domain.labels(), domain.tld());
            }
            Err(e) => println!("  '{}' -> {}", domain_str, e),
        }
    }

    // Example 6: Port validation
    println!("\n=== Port Validation ===");
    let test_ports = [0u16, 22, 80, 443, 1024, 8080, 49152, 65535];

    for port_num in test_ports {
        match Port::new(port_num) {
            Ok(port) => {
                let range = if port.is_privileged() {
                    "privileged/well-known"
                } else if port.is_registered() {
                    "registered"
                } else {
                    "dynamic/ephemeral"
                };
                println!("  {} -> valid ({})", port_num, range);
            }
            Err(e) => println!("  {} -> {}", port_num, e),
        }
    }

    // Example 7: Pure function usage
    println!("\n=== Pure Function Examples ===");
    let ip = Ipv4Addr::new("192.168.1.100".to_string()).unwrap();
    let port = Port::new(8080).unwrap();
    println!("Bind address: {}", format_bind_address(&ip, &port));

    let internal_host = DomainName::new("db.internal".to_string()).unwrap();
    let priv_port = Port::new(443).unwrap();
    println!(
        "Is internal service: {}",
        is_internal_service(&internal_host, &priv_port)
    );
}

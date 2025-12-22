//! Slug validation example demonstrating identifier type usage.
//!
//! This example shows how to use the `Slug` type for URL-safe identifiers,
//! including converting titles to slugs and validating existing slugs.
//!
//! Run with: cargo run --example slug_validation --features full

use stilltypes::prelude::*;
use stillwater::validation::{ValidateAll, Validation};

/// Raw blog post input before validation.
#[derive(Debug)]
struct BlogPostInput {
    title: String,
    custom_slug: Option<String>,
    tags: Vec<String>,
}

/// Validated blog post with guaranteed-valid identifiers.
#[derive(Debug)]
struct ValidBlogPost {
    title: String,
    slug: Slug,
    tags: Vec<Slug>,
}

/// Validates a blog post, accumulating all errors.
fn validate_blog_post(input: BlogPostInput) -> Validation<ValidBlogPost, Vec<DomainError>> {
    // Use custom slug if provided, otherwise generate from title
    let slug_v: Validation<Slug, Vec<DomainError>> = match input.custom_slug {
        Some(custom) => Validation::from_result(Slug::new(custom).map_err(|e| vec![e])),
        None => Validation::from_result(Slug::from_title(&input.title).map_err(|e| vec![e])),
    };

    // Validate all tags as slugs - collect Results first, then convert
    let tag_results: Result<Vec<Slug>, Vec<DomainError>> = input
        .tags
        .into_iter()
        .map(|tag| Slug::from_title(&tag))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| vec![e]);
    let tags_v: Validation<Vec<Slug>, Vec<DomainError>> = Validation::from_result(tag_results);

    (slug_v, tags_v)
        .validate_all()
        .map(|(slug, tags)| ValidBlogPost {
            title: input.title,
            slug,
            tags,
        })
}

/// Pure function - generates blog post URL from validated slug.
fn blog_post_url(slug: &Slug) -> String {
    format!("/posts/{}", slug.get())
}

/// Pure function - generates tag archive URL from validated slug.
fn tag_url(tag: &Slug) -> String {
    format!("/tags/{}", tag.get())
}

fn main() {
    println!("Stilltypes Slug Validation Example");
    println!("===================================\n");

    // Example 1: Valid blog post with auto-generated slug
    println!("=== Valid Blog Post (auto-generated slug) ===");
    let valid_post = BlogPostInput {
        title: "My First Blog Post!".into(),
        custom_slug: None,
        tags: vec!["Rust Programming".into(), "Web Dev".into()],
    };

    match validate_blog_post(valid_post) {
        Validation::Success(post) => {
            println!("Blog post validated successfully!");
            println!("  Title: {}", post.title);
            println!("  Slug: {}", post.slug.get());
            println!("  URL: {}", blog_post_url(&post.slug));
            println!("  Tags:");
            for tag in &post.tags {
                println!("    - {} ({})", tag.get(), tag_url(tag));
            }
        }
        Validation::Failure(errors) => {
            println!("Validation failed!");
            for err in errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 2: Blog post with custom slug
    println!("\n=== Blog Post with Custom Slug ===");
    let custom_slug_post = BlogPostInput {
        title: "A Very Long Title That Would Make an Unwieldy URL".into(),
        custom_slug: Some("short-url".into()),
        tags: vec!["tips".into()],
    };

    match validate_blog_post(custom_slug_post) {
        Validation::Success(post) => {
            println!("Blog post validated successfully!");
            println!("  Title: {}", post.title);
            println!("  Custom Slug: {}", post.slug.get());
            println!("  URL: {}", blog_post_url(&post.slug));
        }
        Validation::Failure(errors) => {
            println!("Validation failed!");
            for err in errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 3: Invalid custom slug
    println!("\n=== Invalid Custom Slug ===");
    let invalid_slug_post = BlogPostInput {
        title: "Good Title".into(),
        custom_slug: Some("Invalid Slug With Spaces".into()),
        tags: vec![],
    };

    match validate_blog_post(invalid_slug_post) {
        Validation::Success(_) => println!("Unexpected success!"),
        Validation::Failure(errors) => {
            println!("Validation failed with {} error(s):", errors.len());
            for err in &errors {
                println!("  - {}", err);
            }
        }
    }

    // Example 4: from_title() conversions
    println!("\n=== Title to Slug Conversions ===");
    let titles = [
        "My First Post",
        "  Spaces  Everywhere  ",
        "Special!@#Characters",
        "Already-a-Slug",
        "123 Numbers First",
        "Unicode Café Test",
    ];

    for title in titles {
        match Slug::from_title(title) {
            Ok(slug) => println!("  '{}' -> '{}'", title, slug.get()),
            Err(e) => println!("  '{}' -> Error: {}", title, e),
        }
    }

    // Example 5: Direct slug validation
    println!("\n=== Direct Slug Validation ===");
    let slugs = [
        "valid-slug",
        "also-valid-123",
        "Invalid Slug",
        "consecutive--hyphens",
        "-leading-hyphen",
        "trailing-hyphen-",
        "",
    ];

    for input in slugs {
        match Slug::new(input.to_string()) {
            Ok(slug) => println!("  '{}' -> valid: {}", input, slug.get()),
            Err(e) => println!("  '{}' -> {}", input, e),
        }
    }

    // Example 6: Generate sitemap entries (pure function)
    println!("\n=== Sitemap Generation (Pure Functions) ===");
    let posts: Vec<(String, Result<Slug, _>)> = vec![
        ("Intro to Rust".into(), Slug::from_title("Intro to Rust")),
        (
            "Advanced Patterns".into(),
            Slug::from_title("Advanced Patterns"),
        ),
        (
            "Web Development".into(),
            Slug::from_title("Web Development"),
        ),
    ];

    println!("  Sitemap entries:");
    for (title, slug_result) in posts {
        if let Ok(slug) = slug_result {
            println!(
                "    <url>https://example.com{}</url> ({})",
                blog_post_url(&slug),
                title
            );
        }
    }
}

<a href='https://ko-fi.com/A0A8Q3SVZ' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi4.png?v=3' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

This repo contains the following main components:

| Crate                                                                                                       | Description                                       |
| ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------- |
| [**saleor-app-sdk**](https://crates.io/crates/saleor-app-sdk)                                               | Types and utilities for making Saleor Apps        |
| [**saleor-app-template**](https://github.com/djkato/saleor-app-template-rs/tree/master/saleor-app-template) | Simple template for making Saleor apps using axum |
| [**saleor-app-sitemap**](https://crates.io/crates/saleor-app-sitemap)                                       | Saleor App for keeping sitemap.xml uptodate       |

# Using this repo

To use, you need to have [Rust environment prepared](https://rustup.rs/).
Every folder represents a different workspace. To add a new lib, do `cargo new <project-name> --lib` or `cargo new <project-name>` for binary apps. It should appear as a new member under root `Cargo.toml`

# Unofficial Saleor App SDK

SDK for building [Saleor Apps](https://github.com/saleor/apps)
to use in your project outside this repo: `cargo add saleor-app-sdk`
to use in your project inside this repo, create a new workspace member and add `saleor-app-sdk.workspace = true` to the members `Cargo.toml`

# Unofficial Saleor App Template

## Creating a new Saleor App from template

If using the `saleor-app-template`, create a new workspace member `cargo new <project-name>`, then `cp saleor-app-template/* <project-name>`.

## Adding new dependencies

Workspace dependencies need to be managed manually. If you wanna add a new dependency to a single member do `cargo add <dep> <project-name>`.
If you want to use a shared dependency, add it to the root level `Cargo.toml`,
then inside your member `Cargo.toml`add it under depencency like: `<dependency> = { workspace = true, features = [ "..." ] }`.

## License

Each workspace member has it's licensed in it's own directory.

### TL;DR:

- saleor-app-sdk, saleor-app-template and the root structure fall under either MIT or Apache 2.0 at your convenience.
- Any other workspace members fall under `FSL-1.1-MIT.md`. If you want to use my apps in commercial environment, each app costs 10€ (or voluntarily more). Upon payment/donation you can automatically use the given app as if it had MIT-1 or Apache 2.0.

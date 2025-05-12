<a href='https://ko-fi.com/A0A8Q3SVZ' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi4.png?v=3' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

# Repo members

| Crate                                                                                                     | Description                                                                                              | License                    | Price | % done |
| --------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | -------------------------- | ----- | ------ |
| [**sdk**](https://crates.io/crates/saleor-app-sdk)                                                        | Types and utilities for making Saleor Apps                                                               | MIT / Apache 2.0           | FOSS  | 90 %   |
| [**app-template**](https://github.com/djkato/saleor-apps-rs/tree/master/app-template)                     | Simple template for making Saleor apps using axum                                                        | MIT / Apache 2.0           | FOSS  | 100 %  |
| [**app-template-ui**](https://github.com/djkato/saleor-apps-rs/tree/master/app-template-ui)               | Advanced template for Saleor apps that work inside the Dashboard, using Leptos (WASM)                    | MIT / Apache 2.0           | FOSS  | 80 %   |
| [**sitemap-generator**](https://github.com/djkato/saleor-apps-rs/tree/master/sitemap-generator)           | Creates and keeps sitemap.txt upto date, no xml support                                                  | PolyForm-Noncommercial-1.0 | 20 €  | 100 %  |
| [**simple-payment-gateway**](https://github.com/djkato/saleor-apps-rs/tree/master/simple-payment-gateway) | Adds payment methods: Cash on delivery, Cash on warehouse pickup, bank tranfer etc.                      | PolyForm-Noncommercial-1.0 | 5 €   | 100 %  |
| [**bulk-price-manipulator**](https://github.com/djkato/saleor-apps-rs/tree/master/bulk-price-manipulator) | Runs a user defined expression to change all variant prices                                              | PolyForm-Noncommercial-1.0 | 10 €  | 100 %  |
| [**heureka-xml-feed**](https://github.com/djkato/saleor-apps-rs/tree/master/heureka-xml-feed)             | Generator for XML Heureka product feed 2.0                                                               | PolyForm-Noncommercial-1.0 | 100 € | 70 %   |
| [**order-analytics**](https://github.com/djkato/saleor-apps-rs/tree/master/order-analytics)               | App with API for order analytics: frequently bought together, best sellers, most bought alternatives etc | PolyForm-Noncommercial-1.0 | 50 €  | 20 %   |

[How do I pay?]()

# Using the apps

To use on bare-metal, clone this repo and just run build and run the apps.

To use with Docker/k8s, you can find prebuilt docker images on the right sidebar next to the code tree under "Packages".
Simply add the package to your `docker-compose.yml`, for example check `docker-compose.yml` file in this repo.

and set all necessary env variables according to the `env.example` file.

# Using this repo

To use, you need to have [Rust environment prepared](https://rustup.rs/).
Every folder represents a different workspace. To add a new lib, do `cargo new <project-name> --lib` or `cargo new <project-name>` for binary apps. It should appear as a new member under root `Cargo.toml`
To run apps propery, use `cargo run -c <crate name>`

## Unofficial Saleor App SDK

SDK for building [Saleor Apps](https://github.com/saleor/apps)
to use in your project outside this repo: `cargo add saleor-app-sdk`
to use in your project inside this repo, create a new workspace member and add `saleor-app-sdk.workspace = true` to the members `Cargo.toml`

## Unofficial Saleor App Template

If using the `app-template`, create a new workspace member `cargo new <project-name>`,`rm -rf <project-name>/*` then `cp -r app-template/* <project-name>/`.
If using the `app-template-ui`, create a new workspace member `cargo new <project-name>`,`rm -rf <project-name>/*` then `cp -r app-template-ui/* <project-name>/`.

### Adding new dependencies

Workspace dependencies need to be managed manually. If you wanna add a new dependency to a single member do `cargo add <dep> --package <project-name>`.
If you want to use a shared dependency, add it to the root level `Cargo.toml`,
then inside your member `Cargo.toml`add it under depencency like: `<dependency> = { workspace = true, features = [ "..." ] }`.

## Developing

To have the app rebuild during development, install bacon `cargo install bacon`, then run `bacon run -- <app-name>` to have bacon watch your code and rerun it on save!
If developing with leptos, use `cargo-leptos` CLI, like `cargo leptos watch`.

## License

Each workspace member has it's license in it's own directory, mentioned in `Cargo.toml`.

### TL;DR:

- saleor-app-sdk, saleor-app-template and the root structure fall under either MIT or Apache 2.0 at your convenience.
- Rest of the apps in this repo fall under `PolyForm-Noncommercial-1.0`. If you want to use my apps commercially, each app costs at least what's written in the [repo members](#repo-members) (or voluntarily more). Upon payment/donation you are allowed to use the given app commercially.

To pay, use either Kofi or github sponsors. If you want to donate / pay directly, email me :)

## Docker images

To build the docker image, log into ghcr.io via docker like `docker login ghcr.io -u <USER> -p <GITHUB KEY WITH PACKAGE PERMS>` run `cargo make`. To publish, run `cargo push-containers`. If you want to push image to your own place, modify `Makefile.toml` and `Dockerfile` to include your username instead of mine.

<a href='https://ko-fi.com/A0A8Q3SVZ' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi4.png?v=3' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

This repo contains the following members:

| Crate                                                                                                                | Description                                                                                         |
| -------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| [**saleor-app-sdk**](https://crates.io/crates/saleor-app-sdk)                                                        | Types and utilities for making Saleor Apps                                                          |
| [**saleor-app-template**](https://github.com/djkato/saleor-apps-rs/tree/master/app-template)                         | Simple template for making Saleor apps using axum                                                   |
| [**saleor-app-sitemap**](https://github.com/djkato/saleor-apps-rs/tree/master/sitemap-generator)                     | Saleor App for keeping sitemap.xml uptodate                                                         |
| [**saleor-app-simple-payment-gateway**](https://github.com/djkato/saleor-apps-rs/tree/master/simple-payment-gateway) | Saleor App that adds payment methods: Cash on delivery, Cash on warehouse pickup, bank tranfer etc. |

# Using the apps

To use on bare-metal, clone this repo and just run build and run the apps.

To use with Docker/k8s, you can find prebuilt docker images on the right sidebar next to the code tree under "Packages".
Simply add the package to your `docker-compose.yml`, for example like so:

```yml
services:
  app-payment-gateway:
    image: ghcr.io/djkato/saleor-simple-payment-gateway:0.1.1
    env_file:
      - docker-gateway.env
    networks:
      - saleor-app-tier
    depends_on:
      - redis-apl
    ports:
      - 3001:3001

  app-sitemap-generator:
    image: ghcr.io/djkato/saleor-sitemap-generator:0.1.0
    env_file:
      - docker-sitemap.env
    networks:
      - saleor-app-tier
    depends_on:
      - redis-apl
    ports:
      - 3002:3002
    volumes:
      - sitemaps:/sitemaps

  redis-apl:
    image: bitnami/redis:latest
    environment:
      - ALLOW_EMPTY_PASSWORD=yes
      - DISABLE_COMMANDS=FLUSHDB,FLUSHALL,CONFIG
    ports:
      - 6380:6379
    networks:
      - saleor-app-tier
    volumes:
      - redis-apl:/bitnami/redis/data

volumes:
  redis-apl:
    driver: local
    driver_opts:
      type: none
      device: ./temp/volumes/redis/
      o: bind
  sitemaps:
    driver: local
    driver_opts:
      type: none
      device: ./temp/docker-sitemaps/
      o: bind

networks:
  saleor-app-tier:
    driver: bridge
```

and set all necessary env variables in `app-simple-gateway.env` according to the `env.example` file.

# Using this repo

To use, you need to have [Rust environment prepared](https://rustup.rs/).
Every folder represents a different workspace. To add a new lib, do `cargo new <project-name> --lib` or `cargo new <project-name>` for binary apps. It should appear as a new member under root `Cargo.toml`
To run apps propery, use `cargo run -c <crate name>`

# Unofficial Saleor App SDK

SDK for building [Saleor Apps](https://github.com/saleor/apps)
to use in your project outside this repo: `cargo add saleor-app-sdk`
to use in your project inside this repo, create a new workspace member and add `saleor-app-sdk.workspace = true` to the members `Cargo.toml`

# Unofficial Saleor App Template

## Creating a new Saleor App from template

If using the `saleor-app-template`, create a new workspace member `cargo new <project-name>`,`rm -rf <project-name>/*` then `cp -r app-template/* <project-name>/`.

## Adding new dependencies

Workspace dependencies need to be managed manually. If you wanna add a new dependency to a single member do `cargo add <dep> --package <project-name>`.
If you want to use a shared dependency, add it to the root level `Cargo.toml`,
then inside your member `Cargo.toml`add it under depencency like: `<dependency> = { workspace = true, features = [ "..." ] }`.

## Developing

To have the app rebuild during development, install bacon `cargo install bacon`, then run `bacon run -- <app-name>` to have bacon watch your code and rerun it on save!

## License

Each workspace member has it's license in it's own directory, mentioned in `Cargo.toml`.

### TL;DR:

- saleor-app-sdk, saleor-app-template and the root structure fall under either MIT or Apache 2.0 at your convenience.
- Rest of the apps in this repo fall under `PolyForm-Noncommercial-1.0`. If you want to use my apps commercially, each app costs 10€ (or voluntarily more). Upon payment/donation you are allowed to use the given app commercially.

## Docker images

To build the docker image, log into ghcr.io via docker like `docker login ghcr.io -u <USER> -p <GITHUB KEY WITH PACKAGE PERMS>` run `cargo make`. To publish, run `cargo push-containers`. If you want to push image to your own place, modify `Makefile.toml` and `Dockerfile` to include your username instead of mine.

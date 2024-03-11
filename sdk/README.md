# Unofficial Saleor App SDK

SDK for building [Saleor Apps](https://github.com/saleor/apps), inspired by The [Official Saleor SDK](https://github.com/saleor/apps)

This repo is very likely to introduce breaking changes as it's very early in development. Made specifically for the [Saleor App Template for Rust](https://github.com/djkato/saleor-apps-rs)

Current Coverage: ~10%

- [x] Base Types (Manifest, Webhooks, SaleorApp, Auth etc.)
- [x] APLs (Only redis currently implemented)
- [x] Webhook utilities (Axum middleware for payload signature verification)
- [ ] JWT Management
- [ ] Settings Manager
- [ ] App Bridge
- [ ] Handlers

## Usage

Check the git repo for example use in saleor-app-template

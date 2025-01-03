# Unofficial Saleor App SDK

SDK for building [Saleor Apps](https://github.com/saleor/apps), inspired by The [Official Saleor SDK](https://github.com/saleor/apps)

This repo is very likely to introduce breaking changes as it's early in development. Made specifically for the [Saleor App Template for Rust](https://github.com/djkato/saleor-apps-rs)

Current Coverage: ~80%

- [x] Base Types (Manifest, Webhooks, SaleorApp, Auth etc.)
- [x] APLs (Only redis and file apl currently implemented)
- [x] Webhook utilities (Axum middleware for payload signature verification)
- [x] JWT Management
- [ ] Settings Manager (in progress rn)
- [x] App Bridge (uses web_sys, wasm_bindgen and similar, is front-end framework agnostic)
- [x] Handlers

## Usage

Check the git repo for example use in app-template or app-template-ui

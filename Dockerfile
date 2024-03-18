FROM rust:latest as chef
RUN apt-get update -y && \
  apt-get install -y pkg-config libssl-dev
# ENV OPENSSL_DIR=/usr
RUN rustup default nightly
RUN cargo install cargo-chef
WORKDIR /apps

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /apps/recipe.json recipe.json
#--target=x86_64-unknown-linux-musl
RUN cargo chef cook --release --recipe-path=recipe.json
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim as chef-sitemap-generator
COPY --from=builder /apps/target/release/sitemap-generator /sitemap-generator
RUN apt-get update -y && \
  apt-get install -y pkg-config libssl-dev curl
RUN mkdir /sitemaps
CMD [ "./sitemap-generator" ]
LABEL service=chef-sitemap-generator
LABEL org.opencontainers.image.title="djkato/saleor-sitemap-generator"\
      org.opencontainers.image.description="Creates and keeps Sitemap.xml uptodate with Saleor." \
      org.opencontainers.image.url="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.source="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.authors="Djkáťo <djkatovfx@gmail.com>"\
      org.opencontainers.image.licenses="PolyForm-Noncommercial-1.0.0"

FROM debian:bookworm-slim as chef-simple-payment-gateway
COPY --from=builder /apps/target/release/simple-payment-gateway /simple-payment-gateway
RUN apt-get update -y && \
  apt-get install -y pkg-config libssl-dev curl
CMD [ "./simple-payment-gateway" ]
LABEL service=chef-simple-payment-gateway
LABEL org.opencontainers.image.title="djkato/saleor-simple-payment-gateway"\
      org.opencontainers.image.description="Payment gateway that adds payment methods that don't need actual verification: Cash on delivery, Cash on warehouse pickup, bank tranfer." \
      org.opencontainers.image.url="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.source="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.authors="Djkáťo <djkatovfx@gmail.com>"\
      org.opencontainers.image.licenses="PolyForm-Noncommercial-1.0.0"

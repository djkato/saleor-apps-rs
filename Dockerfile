FROM rust:latest as chef
RUN apt-get update -y && \
      apt-get install -y pkg-config libssl-dev
# ENV OPENSSL_DIR=/usr
RUN rustup default nightly \
      && cargo install cargo-chef
WORKDIR /apps

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /apps/recipe.json recipe.json
#--target=x86_64-unknown-linux-musl
RUN cargo chef cook --release --recipe-path=recipe.json 
#--bins sitemap-generator bulk-price-manipulator simple-payment-gateway
COPY . .
RUN cargo build --release --package sitemap-generator --package bulk-price-manipulator --package simple-payment-gateway


FROM debian:bookworm-slim as chef-sitemap-generator
WORKDIR /app
COPY --from=builder /apps/target/release/sitemap-generator .
COPY ./sitemap-generator/public ./public
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
WORKDIR /app
COPY --from=builder /apps/target/release/simple-payment-gateway .
COPY ./simple-payment-gateway/public ./public
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

FROM debian:bookworm-slim as chef-bulk-price-manipulator
WORKDIR /app
COPY --from=builder /apps/target/release/bulk-price-manipulator .
COPY ./bulk-price-manipulator/public ./public
RUN apt-get update -y && \
      apt-get install -y pkg-config libssl-dev curl
CMD [ "./bulk-price-manipulator" ]
LABEL service=chef-simple-payment-gateway
LABEL org.opencontainers.image.title="djkato/saleor-bulk-price-manipulator"\
      org.opencontainers.image.description="Saleor App which Runs a user defined expression to change all variant prices" \
      org.opencontainers.image.url="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.source="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.authors="Djkáťo <djkatovfx@gmail.com>"\
      org.opencontainers.image.licenses="PolyForm-Noncommercial-1.0.0"

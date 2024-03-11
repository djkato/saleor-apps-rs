FROM rust:alpine as chef
RUN apk add musl-dev pkgconfig openssl openssl-dev
ENV OPENSSL_DIR=/usr
# RUN rustup default nightly
# RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install cargo-chef
WORKDIR /src

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --target=x86_64-unknown-linux-musl --release --recipe-path=recipe.json
COPY . .
RUN cargo build --target=x86_64-unknown-linux-musl --release

FROM scratch as chef-sitemap-generator
COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/sitemap-generator /sitemap-generator
CMD [ "/sitemap-generator" ]
LABEL service=chef-sitemap-generator
LABEL org.opencontainers.image.title="djkato/saleor-sitemap-generator"\
      org.opencontainers.image.description="Creates and keeps Sitemap.xml uptodate with Saleor." \
      org.opencontainers.image.url="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.source="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.authors="Djkáťo <djkatovfx@gmail.com>"\
      org.opencontainers.image.licenses="PolyForm-Noncommercial-1.0.0"

FROM scratch as chef-simple-payment-gateway
COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/simple-payment-gateway /simple-payment-gateway
CMD [ "/simple-payment-gateway" ]
LABEL service=chef-simple-payment-gateway
LABEL org.opencontainers.image.title="djkato/saleor-simple-payment-gateway"\
      org.opencontainers.image.description="Payment gateway that adds payment methods that don't need actual verification: Cash on delivery, Cash on warehouse pickup, bank tranfer." \
      org.opencontainers.image.url="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.source="https://github.com/djkato/saleor-apps-rs"\
      org.opencontainers.image.authors="Djkáťo <djkatovfx@gmail.com>"\
      org.opencontainers.image.licenses="PolyForm-Noncommercial-1.0.0"

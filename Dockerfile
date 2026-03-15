FROM rust:latest AS builder

WORKDIR /app

COPY . .
RUN cargo build --release

FROM ubuntu:latest AS runner

RUN apt-get update -qq \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    ca-certificates \
    && apt-get -y clean \
    && apt-get -y autoclean \
    && apt-get -y autoremove \
    && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*.deb

LABEL description="Simple Pokedex server"
LABEL authors="Michele d'Amico"

WORKDIR /app

COPY --from=builder /app/target/release/tl-ex /usr/local/bin
EXPOSE 5000

ENTRYPOINT ["tl-ex"]
FROM rust:latest

RUN apt-get update && apt-get install -y \
  aspell \
  aspell-en \
  libaspell-dev \
  libenchant-2-dev

WORKDIR /usr/src/skyspell

COPY . .

RUN cargo install --locked --path crates/ci

WORKDIR /work

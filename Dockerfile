FROM rust:bullseye as builder

RUN apt-get update && apt-get install -y \
  aspell \
  aspell-en \
  libaspell-dev \
  libenchant-2-dev

WORKDIR /usr/src/skyspell
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo install --locked --path crates/ci


FROM debian:bullseye

RUN apt-get update && apt-get install -y \
  aspell \
  aspell-en \
  libenchant-2-2

COPY --from=builder /usr/src/skyspell/target/release/skyspell-ci /usr/bin
CMD /usr/bin/skyspell-ci run

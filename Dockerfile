FROM rust:slim-trixie AS build

RUN apt -y --update install \
    libenchant-2-dev

COPY Cargo.toml Cargo.lock /skyspell/
COPY crates/ /skyspell/crates

WORKDIR /skyspell
RUN cargo build --release

FROM debian:trixie-slim

RUN apt -y --update install \
    libenchant-2-2

COPY --from=build /skyspell/target/release/skyspell /usr/bin
VOLUME [ "/project" ]
ENTRYPOINT [ "/usr/bin/skyspell", "--lang", "en", "--project-path", "/project"]

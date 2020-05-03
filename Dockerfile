FROM rust:1.43 AS build

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
RUN cargo fetch

RUN cargo install cargo-make

COPY src src
RUN cargo build
COPY tests tests
COPY example-package example-package
COPY Makefile.toml Makefile.toml

RUN cargo make test
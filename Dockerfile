################
##### Builder
FROM rust:1.67.1-alpine as builder
WORKDIR /usr/src
RUN USER=root cargo new cypher-arpg-server

WORKDIR /usr/src/cypher-arpg-server

# Install Bevy Linux dependencies
RUN apk add gcc libc-dev pkgconf libx11-dev alsa-lib-dev eudev-dev

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

#################
# Copy in the sources
COPY . /usr/src/cypher-arpg-server/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/cypher-arpg-server/cypher-game/src/main.rs

# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release

################
##### Runtime
FROM alpine:3.16.0 AS runtime

# Copy application binary from builder image
COPY --from=builder /usr/src/cypher-arpg-server/target/x86_64-unknown-linux-musl/release/cypher-game /usr/local/bin/cypher-arpg

RUN cd /usr/local/bin && mkdir cypher-game

# Copy data dependencies
# ZJ-TODO: don't copy to cypher-game; needs to be fixed in data_manager.rs
COPY --from=builder /usr/src/cypher-arpg-server/cypher-game/assets /usr/local/bin/cypher-game

EXPOSE 5000

# Run the application
CMD ["/usr/local/bin/cypher-arpg", "server"]
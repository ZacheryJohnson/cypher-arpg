FROM rust:1.70-bookworm as builder
# Bevy has special requirements on Linux
RUN apt-get update
RUN apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev

WORKDIR /cypher-server
COPY . .
RUN cargo build --release -p cypher-game --no-default-features --features cypher-game/game_server

FROM debian:bookworm-slim
EXPOSE 5000/udp

RUN mkdir -p /cypher-server/assets/game_data/
COPY --from=builder /cypher-server/target/release/cypher-game /cypher-server/cypher-game
COPY --from=builder /cypher-server/cypher-game/assets/game_data /cypher-server/assets/game_data/

ENV GAME_DATA_PATH="/cypher-server/assets/game_data/"
ENV BIND_ADDR="0.0.0.0:5000"

CMD [ "/cypher-server/cypher-game", "server" ]
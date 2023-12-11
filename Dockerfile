FROM rust:1.74.1

WORKDIR /usr/src/tobe-read-bot

COPY . .

RUN cargo install --path .
CMD tobe-read-bot
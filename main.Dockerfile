FROM rust:1.63.0

WORKDIR /usr/src/app

COPY . .

EXPOSE 8081

RUN cargo build

ENTRYPOINT [ "cargo", "run" ]
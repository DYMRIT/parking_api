# Этап сборки
FROM rust:latest as builder

WORKDIR /app

# Установка необходимых пакетов и копирование исходников
COPY . .

RUN apt-get update && apt-get install -y musl-tools musl-dev pkg-config libssl-dev \
    && rustup target add x86_64-unknown-linux-musl \
    && cargo build --target x86_64-unknown-linux-musl --release

# Этап создания финального образа
FROM debian:latest

WORKDIR /app

# Копирование подготовленного исполняемого файла из этапа сборки
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/parking_api /app/parking_api
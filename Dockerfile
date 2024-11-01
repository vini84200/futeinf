FROM rust:1.82 AS builder

RUN USER=root cargo new --bin fute-list
WORKDIR ./fute-list
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./migration ./migration
RUN cargo build --release
RUN rm src/*.rs

RUN rm ./target/release/deps/fute_list*
ADD . ./
RUN cargo build --release
RUN ls target/release/ -la
RUN pwd

FROM debian:buster-slim

ARG APP=/usr/src/app
EXPOSE 8080
ENV TZ=America/Sao_Paulo \
    APP_USER=appuser

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER

COPY --from=builder /fute-list/target/release/fute-list $APP/app

RUN chown -R $APP_USER:$APP_USER $APP

USER $APP_USER
WORKDIR $APP

CMD ["./app"]
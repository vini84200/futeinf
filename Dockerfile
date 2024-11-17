FROM rust:bookworm AS builder

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

FROM debian:bookworm-slim

ARG APP=/usr/src/app
EXPOSE 8080
ENV TZ=America/Sao_Paulo \
    APP_USER=appuser

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libssl-dev openssl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER

COPY --from=builder /fute-list/target/release/fute-list $APP/app
COPY templates $APP/templates

RUN chown -R $APP_USER:$APP_USER $APP

RUN mkdir ${APP}/db && chown -R $APP_USER:$APP_USER ${APP}/db
VOLUME [ "${APP}/db" ]

USER $APP_USER
WORKDIR $APP

CMD ls -la && ./app
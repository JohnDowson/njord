FROM rust:1.50 AS builder
ADD . ./
RUN cargo build --release

FROM debian:bullseye-slim

ARG NJORD_ADDR=0.0.0.0
ARG NJORD_PORT=8080

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && apt-get install -y procps \
    && rm -rf /var/lib/apt/lists/*

EXPOSE ${NJORD_PORT}
ENV TZ=Etc/UTC \
    APP_USER=njorduser \
    APP=/usr/njord \
    NJORD_ADDR=${NJORD_ADDR} \
    NJORD_PORT=${NJORD_PORT}

RUN groupadd ${APP_USER} && useradd -g ${APP_USER} ${APP_USER} \
    && mkdir -p ${APP}

COPY --chown=${APP_USER}:${APP_USER} --from=builder /target/release/njord-restful ${APP}/njord-restful

USER ${APP_USER}
WORKDIR ${APP}
CMD ["./njord-restful"]
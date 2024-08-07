#########################################################################################
## docker will invoke this file from ../.. dir in order to access code
#########################################################################################
FROM rust:1.79-buster AS builder

# RUN apk --update add openssl-dev musl-dev

ARG BUILD_VERSION=0.1

WORKDIR /src/

COPY ./transcriber/ /src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
      CARGO_APP_VERSION=$BUILD_VERSION cargo build --release --bin sound-keeper
#########################################################################################
FROM debian:buster-slim AS runner

RUN apt-get update && apt-get -y install openssl ca-certificates

ENV UID=999

ARG BUILD_VERSION=0.1

WORKDIR /app
EXPOSE 8000

RUN addgroup --gid $UID app && useradd -r -M -u $UID -g app app   
RUN mkdir -p /app && chown -R app:app /app   

COPY --from=builder /src/target/release/sound-keeper /app/

RUN chown app:app /app/* /app
 
USER app

ENTRYPOINT ["./sound-keeper"]

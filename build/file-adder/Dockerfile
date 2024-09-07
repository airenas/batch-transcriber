#########################################################################################
## docker will invoke this file from ../.. dir in order to access code
#########################################################################################
FROM rust:1.79-buster AS builder

# RUN apt-get update && apt-get -y install openssl libssl-dev

ARG BUILD_VERSION=0.1

WORKDIR /src/

COPY ./transcriber/ /src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
      CARGO_APP_VERSION=$BUILD_VERSION cargo build --release --bin file-adder

#########################################################################################
FROM gcr.io/distroless/cc-debian11 AS ssl
#########################################################################################
FROM debian:buster AS pg-builder
#########################################################################################
RUN apt-get update
RUN apt-get update && apt-get -y install libpq5
#########################################################################################
# Debian 12 does not include ssl libs
FROM gcr.io/distroless/cc-debian12 AS runner
#########################################################################################

COPY LICENSE /licenses/LICENSE-bsd-3

WORKDIR /app

#########################################################################################
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libpq.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libgssapi_krb5.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libldap_r-2.4.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libkrb5.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libk5crypto.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libkrb5support.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/liblber-2.4.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libsasl2.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libgnutls.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libp11-kit.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libidn2.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libunistring.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libtasn1.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libnettle.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libhogweed.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libgmp.so* /usr/lib/x86_64-linux-gnu/
COPY --from=pg-builder /usr/lib/x86_64-linux-gnu/libffi.so* /usr/lib/x86_64-linux-gnu/

### /lib/x86_64-linux-gnu
COPY --from=pg-builder /lib/x86_64-linux-gnu/libcom_err.so.2 /lib/x86_64-linux-gnu/libcom_err.so.2
COPY --from=pg-builder /lib/x86_64-linux-gnu/libcom_err.so.2.1 /lib/x86_64-linux-gnu/libcom_err.so.2.1
COPY --from=pg-builder /lib/x86_64-linux-gnu/libkeyutils.so.1 /lib/x86_64-linux-gnu/libkeyutils.so.1

### /ssl
COPY --from=ssl /usr/lib/x86_64-linux-gnu/libssl.so.* /lib/x86_64-linux-gnu/
COPY --from=ssl /usr/lib/x86_64-linux-gnu/libcrypto.so.* /lib/x86_64-linux-gnu/
#########################################################################################

COPY --from=builder /src/target/release/file-adder /app/

ENTRYPOINT ["./file-adder"]

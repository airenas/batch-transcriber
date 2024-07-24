#########################################################################################
## docker will invoke this file from ../.. dir in order to access code
#########################################################################################
FROM jameshiew/diesel-cli:2.0.1

ARG BUILD_VERSION=0.1

LABEL org.opencontainers.image.version=$BUILD_VERSION \
      org.opencontainers.image.authors="airenass@gmail.com" \
      name="bt-dbmigrate" \
      maintainer="airenass@gmail.com" \
      vendor="airenass@gmail.com" \
      version=$BUILD_VERSION \
      release=$BUILD_VERSION \
      summary="This image is used to migrate bt db" \
      description="This image is used to migrate bt db" 

COPY transcriber/migrations /migrations/

ENTRYPOINT ["diesel"]

CMD ["migration", "run", "--migration-dir" , "/migrations"]

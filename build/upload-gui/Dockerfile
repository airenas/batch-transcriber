FROM node:22-alpine3.19 AS base

FROM base AS deps
RUN apk add --no-cache libc6-compat

WORKDIR /app

COPY upload-gui/package.json upload-gui/package-lock.json* ./
RUN npm ci


# 2. Rebuild the source code only when needed
FROM base AS builder
ARG BUILD_VERSION=0.1
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY upload-gui/ .
RUN echo "export const VERSION = '$BUILD_VERSION'" > /app/app/version.ts
RUN npm run build

#############################################################################
FROM nginxinc/nginx-unprivileged:1.27-alpine3.19

COPY build/upload-gui/entrypoint.sh /root/entrypoint.sh
COPY build/upload-gui/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/out /usr/share/nginx/html

USER root

# RUN chown nginx:nginx /root/entrypoint.sh
RUN chown nginx:nginx /usr/share/nginx/html/*

USER nginx

# CMD ["/root/entrypoint.sh"]

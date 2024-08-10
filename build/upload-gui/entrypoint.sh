#!/bin/sh
set -e
echo "Set base path to '$BASE_PATH'"
find /usr/share/nginx/html -type f -exec sed -i "s|/__BASE_PATH_/|$BASE_PATH|g" {} +
echo "Set base path to '$BASE_PATH' done"

echo "Set SERVER_URL to '$SERVER_URL'"
# grep -R __SERVER_URL__  /usr/share/nginx/html/*
find /usr/share/nginx/html -type f -exec sed -i "s|__SERVER_URL__|$SERVER_URL|g" {} +
echo "Set SERVER_URL url to '$SERVER_URL' done"

# echo Starting nginx
# exec nginx -g 'daemon off;'

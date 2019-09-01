#!/bin/bash

# Ensure folders are owned by www-data when
# mounted on subfolders
chown -R www-data:www-data /var/www/html

docker-entrypoint.sh "$@"
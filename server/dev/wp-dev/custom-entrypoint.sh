#!/bin/bash

function fix_linux_internal_host() {
DOCKER_INTERNAL_HOST="host.docker.internal"
    if ! grep $DOCKER_INTERNAL_HOST /etc/hosts > /dev/null ; then
        DOCKER_INTERNAL_IP=$(/sbin/ip route | awk '/default/ { print $3}' | awk '!seen[$0]++')
    #DOCKER_INTERNAL_IP='/sbin/ip route | awk '/default/ { print $3 }' | awk '!seen[$0]++'
        echo -e "$DOCKER_INTERNAL_IP\t$DOCKER_INTERNAL_HOST" | tee -a /etc/hosts > /dev/null
        echo "Added $DOCKER_INTERNAL_HOST to hosts /etc/hosts"
        cat /etc/hosts
    fi
}

fix_linux_internal_host
touch /tmp/xdebug_remote.log
chown www-data:www-data /tmp/xdebug_remote.log

# Ensure folders are owned by www-data when
# mounted on subfolders
chown -R www-data:www-data /var/www/html

docker-entrypoint.sh "$@"
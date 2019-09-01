#!/bin/bash
if [ -z "$1" ]; then
    echo "Doing restore to commit: $1"
else
    echo "Doing restore to latest commit"
fi

#cat backup/backup.sql | docker-compose exec -T db /usr/bin/mysql --login-path=local wordpress

source dev.sh run backup ./opt/restore.sh "$@"

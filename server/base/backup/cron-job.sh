#!/bin/bash

. /opt/lib.sh

wait_for_lock_file

touch "$LOCK_FILE"

ensure_git_repo

# compare last modification in db_data to last mysql-dump.sql
# storing modification in a dedicated file
MYSQLDUMP_DATE_FILE="$VAR_DIR/mysqldump.date"
# Possibly better way to check for updates is some query we could also do
# which would also not require access to db_data
# LAST_MYSQL_MODIFICATION=$(find "$VAR_DIR/db_data" -type f -exec date -r "{}" +%Y%m%d%H%M%S \; | sort -r | head -n 1)
LAST_MYSQL_MODIFICATION=$(mysql -u wordpress -pwordpress -h db -e '
SELECT DATE_FORMAT(MAX(UPDATE_TIME), "%Y%m%d%H%i%S") FROM `TABLES`' INFORMATION_SCHEMA | tail -n 1)
LAST_DUMP=$(
    if [ -f "$MYSQLDUMP_DATE_FILE" ]
    then
        cat $MYSQLDUMP_DATE_FILE
    else
        echo "0"
    fi
)
echo "Last modification: $LAST_MYSQL_MODIFICATION, last dump: $LAST_DUMP"
# Seems mysql returns null when not changed in this session
# Could force some dumps at intervals
if [ $LAST_MYSQL_MODIFICATION != "NULL" ] && [ $LAST_MYSQL_MODIFICATION \> $LAST_DUMP ]
then
    echo "Newer, making new mysqldump"
    # --compact would be nice, but could use delete table for restore
    # todo: adjust output
    mysqldump -u wordpress -pwordpress -h db --result-file="$MYSQLDUMP_FILE" wordpress
    echo $(date +%Y%m%d%H%M%S) > $MYSQLDUMP_DATE_FILE
fi

# pulling from git, but keeping local changes,
# I think overwrite should be explicit, but
# getting into muddy waters

# Possibly do this only when changes (I think the most
# important is to make updating robust, so
# not blocked on remote updates)
cd "$VAR_DIR/repo"

# now check if git status reports changes,
# if so then push changes
GIT_STATUS=$(git status --porcelain)
if [ "$GIT_STATUS" != ""  ]; then
  # Changes
  echo "Changes detected, making commit"
  echo "$GIT_STATUS"
  push_git "Cron backup"
else
  echo "No changes detected"
  # Possibly some reasons to keep repo updated from master,
  # but should probably be explicit
  # git pull
fi

rm "$LOCK_FILE"
#!/bin/bash

GIT_USER="brygga-dev"
GIT_EMAIL="52463886+brygga-dev@users.noreply.github.com"
GIT_REPO="https://brygga-dev@github.com/brygga-dev/brygga-backup.git"

VAR_DIR="/var/lib/docker-backup"
MYSQLDUMP_FILE="$VAR_DIR/repo/mysqldump.sql"

LOCK_FILE="$VAR_DIR/.lock"

# Lock file to avoid clash with restore
# Or ignore if more than 10 minutes
wait_for_lock_file()
{
    if [ -f $LOCK_FILE ]; then
        lastModificationSeconds=$(date +%s -r "$LOCK_FILE")
        currentSeconds=$(date +%s)
        ignoreAt=$((currentSeconds - 600))
        if [ $lastModificationSeconds -lt $ignoreAt ]; then
            waits=0
            while [ -f $LOCK_FILE ]; do
                    if [ $waits -eq 10 ]; then
                            echo "Timeout waiting for lock file"
                            exit 1
                    fi
                    sleep 2
                    waits=$((waits + 1))
                    echo "Waiting for lock $waits/10"
            done
        fi
    fi
}

# Ensures the permissions on repo is right
ensure_repo_permissions()
{
    chown -R 33:33 "$VAR_DIR/repo/uploads"
    chmod -R 0755 "$VAR_DIR/repo/uploads"
}

restore_db()
{
    if [ -f "$MYSQLDUMP_FILE" ]; then
        echo "Restoring database"
        mysql -u wordpress -pwordpress -h db  wordpress < "$MYSQLDUMP_FILE"
    else
        echo "No dump file to restore, leaving as is, resetting not implemented"
    fi
}

# uses folders:
# - /var/lib/docker-backup
# - /var/lib/docker-backup/init_repo    Initial fetch of repo, which is then moved to ../repo
# - /var/lib/docker-backup/repo         Main location of backup files, upload and mysqldump.sql is here

# Setup git repo if not already
ensure_git_repo()
{
    if [ ! -d "$VAR_DIR/repo/.git" ]; then
        echo "Initializing repo"
        # todo: Check return status code
        git clone --depth 1 $GIT_REPO "$VAR_DIR/init_repo"
        cd "$VAR_DIR/init_repo"
        git config user.name $GIT_USER
        git config user.email $GIT_EMAIL
        # Move with update flag that only
        # replaces if newer
        echo "Moving files from git"
        mv -n * ../repo/

        # Permissions
        ensure_repo_permissions

        # Assuming this is initial run, restore database
        restore_db

        mv .git ../repo/
        cd ..
        rm -rf init_repo
    fi
}

push_git()
{
  # First make sure repo is updated for robustness
  # todo: check better vs origin
  echo "Stashing"
  git add .
  git stash > /dev/null
  echo "Pulling"
  git pull
  echo "Pop"
  #git stash pop > /dev/null
  #https://stackoverflow.com/questions/16606203/force-git-stash-to-overwrite-added-files
  git checkout stash -- .

  # Add, commit and push
  echo "Add"
  # I think not needed as I did it before stash now
  git add .
  echo "Commit"
  git commit -m "$1 $(date)"
  echo "Push"
  git push origin master
}

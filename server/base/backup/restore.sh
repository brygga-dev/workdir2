#!/bin/bash

# Passing and argument will clone a certain
# commit, and set it up as the contents of
# the `repo` folder, then run the mysqldump
# from the commit

. /opt/lib.sh

wait_for_lock_file

touch "$LOCK_FILE"

ensure_git_repo

# Cloning repo, this adds the checkout as a new
# commit, but there should be better ways to do this
# probably..
# Not sure git reset --hard is good as keeping the history
# can be an advantage
echo "Cloning repo"
git clone $GIT_REPO "$VAR_DIR/restore_repo"
cd "$VAR_DIR/restore_repo"

# Check if a has was provided as argument
if [ ! -z "$1" ]; then
    git checkout $1
fi

rm -rf .git
# Just replacing all files. Possibly some option to "merge",
# but doesn't always make sense, there wouldn't be database
# entries any more to other files.
# todo, possibly could do a stash, pull, pop, push for
# any last changes. Would be nice with code sharing
echo "Restoring files from git"

# Move .git out so we can simply move repository file
#if [ -d "$VAR_DIR/repo/.git" ]; then
#    echo "Moving .git out of /repo/"
#    mv "$VAR_DIR/repo/.git" "$VAR_DIR/.git"
#fi

# In dev mode, uploads is currently mounted, so
# it's delete gives errors.

# Deleting current files in `repo`
echo "Current uploads"
ls "$VAR_DIR/repo/uploads"

rm -rf "$VAR_DIR/repo/uploads/"*

echo "Removed uploads"
ls "$VAR_DIR/repo/uploads"

# Moving the cloned/restored files to `repo` folder
if [ -d "uploads" ]; then
    echo "Moving uploads"
    mv uploads/* "$VAR_DIR/repo/uploads/"

    echo "After move"
    ls "$VAR_DIR/repo/uploads"
fi
if [ -f "mysqldump.sql" ]; then
    echo "Moving mysqldump"
    rm -f "$MYSQLDUMP_FILE"
    mv mysqldump.sql "$MYSQLDUMP_FILE"
fi

# Permissions
ensure_repo_permissions

# Then moving the .git in again
#if [ -d "$VAR_DIR/.git" ]; then
#    echo "Moving .git back to /repo/"
#    mv "$VAR_DIR/.git" "$VAR_DIR/repo/.git"
#fi

# Now deleting the empty restore folder
cd ..
rm -rf restore_repo

cd "$VAR_DIR/repo"

# Now run mysqldump file
restore_db

# TODO: This could be from an earlier wp version, so we should run wp upgrade/migrate

# Quick commit, todo: modularize to some functions
push_git "Restore to $1"

rm "$LOCK_FILE"

echo "Restored from commit $1"
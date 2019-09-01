#!/bin/bash
source wp.sh core download
source wp.sh core install \
    --url=wordpress-container \
    --title=Wordpress \
    --admin_user=admin \
    --admin_password=pass \
    --admin_email=brygga.dev@gmail.com \
    --skip-email

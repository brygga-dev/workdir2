#!/bin/bash

# Prod with backup

if [ $# -eq 0 ]; then
	docker-compose \
		-f docker-compose.yml \
		-f docker-compose.prod.yml \
		-f docker-backup.yml \
		up
else
	docker-compose \
		-f docker-compose.yml \
		-f docker-compose.prod.yml \
		-f docker-backup.yml \
		"$@"
fi

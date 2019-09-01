#!/bin/bash

# Prod without auto backup

if [ $# -eq 0 ]; then
	docker-compose \
		-f docker-compose.yml \
		-f docker-compose.prod.yml \
		up
else
	docker-compose \
		-f docker-compose.yml \
		-f docker-compose.prod.yml \
		"$@"
fi

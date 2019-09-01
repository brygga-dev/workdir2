#!/bin/bash

if [ $# -eq 0 ]; then
	docker-compose \
		-f docker-compose.yml \
		-f docker-compose.dev.yml \
		-f docker-reimage.yml \
		up
else
	docker-compose \
		-f docker-compose.yml \
		-f docker-compose.dev.yml \
		-f docker-reimage.yml \
		"$@"
fi

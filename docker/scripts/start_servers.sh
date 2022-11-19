#! /bin/bash
docker-compose --file ./docker/local_server/docker-compose.yml --env-file ./.env up -d --force-recreate
docker-compose --file ./docker/squid/docker-compose.yml up -d --force-recreate
#! /bin/bash
docker compose --file ./docker/local_server/docker-compose.yml --env-file ./.env down
docker compose --file ./docker/squid/docker-compose.yml down
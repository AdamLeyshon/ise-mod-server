#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE ise;
    CREATE ROLE ise_api with PASSWORD '#8fSUK)vmFEpGtuv' LOGIN;
    GRANT ALL PRIVILEGES ON DATABASE ise TO ise_api;
EOSQL

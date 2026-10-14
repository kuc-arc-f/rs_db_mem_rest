#!/bin/bash

#export TOKIO_WORKER_THREADS=4

sqlite3 ./data/backup.db .dump > ./data/backup.sql

./target/release/db_mem_api_server

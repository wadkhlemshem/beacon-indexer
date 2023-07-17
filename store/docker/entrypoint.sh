#!/usr/bin/bash

set -x

diesel migration run --database-url $DATABASE_URL --migration-dir /migrations

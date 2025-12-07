#!/bin/bash
set -e

echo "Waiting for Postgres to be ready..."
# Loop until Postgres is ready
# We use pg_isready which comes with postgresql-client
# We need to extract host/port/user from DATABASE_URL or rely on env vars if set differently.
# Ideally, we just try to run migrate in a loop or use a wait-for-it script.
# Simple retry loop for migration is robust enough.

until ./migrate; do
  echo "Migration failed (database might be initializing), sleeping..."
  sleep 2
done

echo "Migrations completed."

echo "Running seeder..."
./seed-cli

echo "Starting server..."
./coding-quiz-api

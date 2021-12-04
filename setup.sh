#!/bin/bash
set -eux

# Check command line variable
if [ $# -ne 2 ]; then
  echo "Error: $# arguments were given." 1>&2
  echo "2 arguments are needed." 1>&2
  exit 1
fi

# Parameters
# MAINNAME='reservoir'
DBNAME='reservoir'
USERNAME=$1
PASSWORD=$2

# Install PostgreSQL
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo /etc/init.d/postgresql start

# Change postgres password
# sudo su - postgres
# psql -U postgres -d postgres -c "alter role postgres with password '${PASSWORD}'"

# Create new role
psql -U postgres -d ${DBNAME} -c "CREATE ROLE ${USERNAME} WITH LOGIN PASSWORD '${PASSWORD}'"

# Create database
psql -U ${USERNAME} -d postgres -c "CRESTE DATABASE ${DBNAME}"

psql -U ${USERNAME} -d ${DBNAME} -a -f ./sql/create_table_persons.sql
psql -U ${USERNAME} -d ${DBNAME} -a -f ./sql/create_table_resources.sql
psql -U ${USERNAME} -d ${DBNAME} -a -f ./sql/create_table_reservations.sql

# psql -U ${MAINNAME} -d ${MAINNAME} -c "CREATE ROLE ${MAINNAME} WITH LOGIN PASSWORD ${PASSWORD}"
# psql -U postgres -d ${MAINNAME} -c "SELECT c_defaults  FROM user_info WHERE c_uid = 'testuser'"

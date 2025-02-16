#!/bin/bash
set -e
# This script is used to build the development version of the project.
cd "$(dirname "$0")/web"

curl http://localhost:8081/api/openapi.json -o config/openapi.json 
npm run openapi
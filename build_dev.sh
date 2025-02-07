#!/bin/bash
set -e
# This script is used to build the development version of the project.
cd "$(dirname "$0")"
./update_static.sh
cargo build

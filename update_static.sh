#!/bin/bash
set -e
# This script is used to build the development version of the project.
cd "$(dirname "$0")"
pushd "web"
npm run build
popd
rm -rf target/debug/static
cp -r web/dist target/debug/static
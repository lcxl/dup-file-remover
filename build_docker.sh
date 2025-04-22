#!/bin/bash
set -e

help_func() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  -t <tag>   Specify the Docker image tag"
    echo "  -p         Push the Docker image to the registry"
    echo "  -h         Show this help message"
    exit 0
}

cd "$(dirname "$0")"

DFR_IMAGE_TAG=${DFR_IMAGE_TAG:-"lcxl/dup-file-remover:latest"}

PUSH_IMAGE=false

while getopts 't:ph' OPT; do
    case $OPT in
        t) DFR_IMAGE_TAG="$OPTARG";;
        p) PUSH_IMAGE=true;;
        h) help_func;;
        ?) help_func;;
    esac
done



echo "Building Docker image with tag $DFR_IMAGE_TAG"

DOCKER_BUILDKIT=1 docker build -t $DFR_IMAGE_TAG .

if [[ "$PUSH_IMAGE" = true ]]; then
    echo "Push Docker image  $DFR_IMAGE_TAG to registry"
    docker image push $DFR_IMAGE_TAG
fi
#!/bin/bash
set -e

ACTION=${1:-build}

build_typescript() {
    cd typescript
    tsc
    cd ..
}

build_rust() {
    cargo build --release
}

build_python() {
    python -m build
}

publish_typescript() {
    npm publish
}

publish_rust() {
    cargo publish --allow-dirty
}

publish_python() {
    twine upload dist/*
}

clean_all() {
    rm -rf typescript/dist
    cargo clean
    rm -rf dist build *.egg-info python/*.egg-info
}

case "$ACTION" in
    build)
        build_typescript
        build_rust
        build_python
        ;;
    publish)
        publish_typescript
        publish_rust
        publish_python
        ;;
    clean)
        clean_all
        ;;
    release)
        build_typescript
        build_rust
        build_python
        publish_typescript
        publish_rust
        publish_python
        ;;
    *)
        echo "Usage: ./scripts/build.sh [build|publish|clean|release]"
        exit 1
        ;;
esac

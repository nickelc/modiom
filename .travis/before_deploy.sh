mk_artifacts() {
    cargo build --target "$TARGET" --release
}

mk_tarball() {
    local name="${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}"
    local staging="staging/$name"
    local out_dir="$(pwd)/deployment"
    mkdir -p "$staging"
    mkdir -p "$out_dir"

    cp "target/$TARGET/release/modiom" "$staging/modiom"
    strip "$staging/modiom"
    cp {README.md,LICENSE-APACHE,LICENSE-MIT} "$staging/"

    (cd $(dirname $staging) && tar czf "$out_dir/$name.tar.gz" "$name")
}

main() {
    if [ "$TRAVIS_TAG" != "" ]; then
        mk_artifacts
        mk_tarball
    fi
}

main

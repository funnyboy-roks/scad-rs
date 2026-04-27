#!/bin/bash

set -xe

if [ -z $1 ]; then
    echo "Usage: $0 <example-name>" >&2
    exit 1
fi

file=$(mktemp scad-rs.XXXXXXXXXX.scad)
cargo r --example "$1" > "$file" 2>/dev/null
openscad --viewall $file &
os_pid=$?
cargo watch -w src -w examples -- bash -c "cargo r --example '$1' > '$file'" &
w_pid=$?

die () {
    kill $os_pid || true
    kill $w_pid  || true
    rm $file
    exit 0
}

jobs -p

trap die INT
wait

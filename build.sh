#!/usr/bin/env bash

echo "--- building 'ulp' ---"
cd ulp
cargo build

echo "--- building 'main' ---"
cd ../main
cargo build --features native

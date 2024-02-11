#!/bin/sh

cd app/ && cargo test --features CI && [ $? -eq 0 ] || exit 1

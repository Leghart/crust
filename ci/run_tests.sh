#!/bin/sh

cd app/ && cargo test && [ $? -eq 0 ] || exit 1

#!/bin/sh

export RUST_LOG=info

exec ./aarch64/pythonbox \
  --port=7075 \
  --image="python:alpine"

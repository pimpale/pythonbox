#!/bin/sh

exec ./aarch64/pythonbox \
  --port=9075 \
  --image="python:alpine"

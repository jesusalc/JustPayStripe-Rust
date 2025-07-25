#!/bin/bash

if command -v miniserve >/dev/null 2>&1; then
{
  cargo install miniserve
}
fi

function main() {

  cd examples/frontend
  miniserve . --index index.html --port 3000
}

main



#

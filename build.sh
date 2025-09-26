#!/usr/bin/env bash

if [[ "$(basename "$(pwd)")" == "daily-bugle" ]]; then
  echo "✅ In daily-bugle project directory"
else
  echo "❌ Not in daily-bugle"
  exit 1
fi

if [[ ! -d .git ]]; then
  echo "❌ This is not the project root (no .git directory)."
  exit 1
else
  echo "✅ In the correct project directory"
fi

function info() {
  echo -e "\033[1;36m💡 INFO:\033[0m $1"
}

info "Building cli"
cargo build
info "Building supporting node module"
cd google
pnpm build

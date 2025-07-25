#!/bin/bash

set -e

echo "🔄 Running tests..."
cargo test

echo "📦 Publishing to crates.io..."
cargo publish

echo "✅ Done!"

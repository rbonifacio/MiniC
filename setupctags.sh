#!/bin/bash
## Setup ctags for nom, rust-std crates and project
set -e  # Exit on error

## check existence of stdlib
if [ ! -d rust ]; then
    echo "Cloning Rust source..."
    git clone --depth 1 https://github.com/rust-lang/rust.git
fi

## create stdlib tags
if [ -d rust ]; then
    echo "Generating stdlib tags..."
    (cd rust && ctags -R --languages=Rust -f ~/rust-stdlib.tags library/)
    echo "✅ Created ~/rust-stdlib.tags"
fi

## create project tags
echo "Generating project tags..."
ctags -R --languages=Rust --exclude=target -f tags .
echo "✅ Created ./tags"

## find and tag nom (if exists)
NOM_PATH=$(find ~/.cargo/registry/src -type d -name "nom-*" 2>/dev/null | head -1)
if [ -n "$NOM_PATH" ]; then
    echo "Generating nom tags..."
    ctags -R --languages=Rust -f nom.tags "$NOM_PATH/src"
    echo "✅ Created nom.tags"
else
    echo "⚠️  nom not found in cargo cache (build your project first?)"
fi

## find and tag proptest (if exists)
PROPTEST_PATH=$(find ~/.cargo/registry/src/index.crates.io-* -type d -name "proptest-*" 2>/dev/null | head -1)
if [ -n "$PROPTEST_PATH" ]; then
    echo "Generating proptest tags..."
    ctags -R --languages=Rust -f proptest.tags "$PROPTEST_PATH/src"
    echo "✅ Created proptest.tags"
else
    echo "⚠️  proptest not found in cargo cache (build your project first?)"
fi

echo ""
echo "🎉 Done! Add to Vim: :set tags=./tags,~/rust-stdlib.tags"

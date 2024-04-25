#!/usr/bin/env bash

rev=$(git rev-parse HEAD)

sed -i "s/rev = \".*\";/rev = \"$rev\";/" README.md

sha=$(nix-prefetch-git https://github.com/unixpariah/ssb.git | grep sha256 | head -1 | awk '{print $2 }' | tr -d '",')

sed -i "s/sha256 = \".*\";/sha256 = \"$sha\";/" README.md

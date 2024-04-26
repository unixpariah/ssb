#!/usr/bin/env bash

rev=$(nix-prefetch-git https://github.com/unixpariah/waystatus.git | grep rev | awk '{print $2 }' | tr -d '",')

sed -i "s/rev = \".*\";/rev = \"$rev\";/" README.md

sha=$(nix-prefetch-git https://github.com/unixpariah/waystatus.git | grep sha256 | head -1 | awk '{print $2 }' | tr -d '",')

sed -i "s/sha256 = \".*\";/sha256 = \"$sha\";/" README.md

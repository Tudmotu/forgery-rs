#!/usr/bin/env bash

if ! command -v cargo &> /dev/null; then
    echo 'Rust and Cargo are necessary to install Forgery';
    exit 127;
fi

if command -v forgery &> /dev/null; then
    echo 'A Forgery version already exists, reinstalling...';
fi

CACHE_PATH=~/.cache/forgery-rs/
FORGERY_REPO=https://github.com/Tudmotu/forgery-rs.git

echo 'Installing Forgery from source...';
mkdir -p $CACHE_PATH
pushd $CACHE_PATH
git clone $FORGERY_REPO repo
pushd repo
cargo install --locked --path .
if [ $? -eq 0 ]; then
    if command -v forgery &> /dev/null; then
        echo "Forgery was installed at: $(command -v forgery)";
    else
        echo 'Forgery was installed.';
        echo "Try adding cargo's bin directory to your PATH";
    fi
else
    echo 'An error occured during Forgery installation';
fi
popd
rm -rf $CACHE_PATH/repo
popd

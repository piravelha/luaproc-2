#!/usr/bin/bash

set -xe

cargo build --release && cp target/release/luaproc ./luaproc

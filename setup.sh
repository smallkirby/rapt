#!/bin/bash

cargo build --release
mkdir lists
mkdir archive
mkdir apt
ln -s $(pwd)/target/release/rapt $(pwd)/rapt
./rapt update

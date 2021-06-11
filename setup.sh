#!/bin/bash

cargo build --release
mkdir lists
mkdir archive
ln -s $(pwd)/target/release/rapt $(pwd)/rapt
./rapt update

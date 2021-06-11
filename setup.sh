#!/bin/bash

cargo build --release
mkdir lists
mkdir archives
ln -s $(pwd)/target/release/rapt $(pwd)/rapt
./rapt update

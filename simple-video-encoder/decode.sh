#!/usr/bin/bash

rm ../data/decoded*
cargo build && ./target/debug/simple-video-encoder -i ../data/encoded.rle -o ../data/ -d
#cargo build --release && ./target/release/simple-video-encoder -i ../data/encoded.rle -o ../data/ -d

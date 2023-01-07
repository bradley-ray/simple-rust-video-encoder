#!/usr/bin/bash

rm ../data/encoded*
cargo build && ./target/debug/simple-video-encoder -i ../data/video.rgb24 -o ../data/
#cargo build --release && ./target/release/simple-video-encoder -i ../data/video.rgb24 -o ../data/

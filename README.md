# Simple Rust Video Encoder

Implementation of the simple video encoder from [kevmo314](https://github.com/kevmo314/codec-from-scratch)
For educational purposeses, to learn Rust and a little bit about video encoding.

## TODO
- [x] get file read/write to work
- [x] get rgb to yuv conversion to work 
- [x] write encoder/decoder (rle)
  - [x] fix weird noise in encoder/decoder
- [ ] refactor
  - [x] issues with speed compared to kevmo314's go implementation
  - [ ] attempt to reduce all type Vec<Vec<u8>> to just type Vec<u8>
  - [ ] can improve readability a lot along with scalability for later codec implmentations if wanted
  - [x] needs better cmd line argument interface
  - [x] need to improve error handling

## Maybe Later TODO
* implement DEFLATE
* implement a more advanced/modern codec

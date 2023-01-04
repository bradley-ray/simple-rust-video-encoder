# Simple Rust Video Encoder

Implementation of the simple video encoder from [kevmo314](https://github.com/kevmo314/codec-from-scratch)
For educational purposeses, to learn Rust and a little bit about video encoding.

## TODO
- [x] get file read/write to work
- [x] get rgb to yuv conversion to work 
- [x] write encoder/decoder (rle)
  - [x] fix weird noise in encoder/decoder
- [ ] refactor
  * issues with speed compared to kevmo314's go implementation
  * can improve readability a lot along with scalability for later codec implmentations if wanted
  * needs better cmd line argument interface
  * need to improve error handling

## Maybe Later TODO
* implement DEFLATE
* implement a more advanced/modern codec

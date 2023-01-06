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
    * i'm going to count this as done (minus some small imporvements I'll make with regards to memory allocation)
    * building for release gives me speed a lot more than the go implementation
    * from quick reading, build for release better optimizers iterations which is where
      I was having all my issues
  - [ ] can improve readability a lot along with scalability for later codec implmentations if wanted
  - [x] needs better cmd line argument interface
  - [x] need to improve error handling

## Maybe Later TODO
* implement DEFLATE
* implement a more advanced/modern codec

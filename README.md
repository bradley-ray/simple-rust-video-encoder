# Simple Rust Video Encoder

Implementation of the simple video encoder from [kevmo314](https://github.com/kevmo314/codec-from-scratch)
For educational purposeses, to learn Rust and a little bit about video encoding.

## TODO
- [x] get file read/write to work
- [x] get rgb to yuv conversion to work 
- [x] write encoder/decoder (rle)
  - [x] fix weird noise in encoder/decoder
- [ ] refactor
  - [ ] issues with speed compared to kevmo314's go implementation
    * really struggling to determine what the specific slow downs are (compared to go implementation from kevmo314)
    * the issue really seems to be certain loops causing like 75% of the slowdown, 
      but these loops are not really doing much compared to the ones where I am reallocating vectors each time
    * weirdly enough, moving the vectors so they are allocated just once still doesn't improve performance (noticeably)
    * so for now, will move on, and maybe return in the future to figure out the issue
  - [ ] can improve readability a lot along with scalability for later codec implmentations if wanted
  - [x] needs better cmd line argument interface
  - [x] need to improve error handling

## Maybe Later TODO
* implement DEFLATE
* implement a more advanced/modern codec

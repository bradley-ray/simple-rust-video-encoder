use std::env;
use std::fs;

use std::io::{Read, BufReader};
use std::io::{Write, BufWriter};

const MB: f32 = 1_000_000.;

// TODO: currently just lazily using unwrap, so need to
// properly handle the Result<> later
fn main() {
    // TODO: make/use better tooling for cmd line args
    let args: Vec<String> = env::args().collect();
    let in_path = args.get(1).unwrap();
    let out_path = args.get(2).unwrap();
    let width: usize = args.get(3).unwrap().parse().unwrap();
    let height: usize = args.get(4).unwrap().parse().unwrap();

    println!("input path: {}", in_path);
    println!("output path: {}", out_path);
    println!("width: {}", width);
    println!("height: {}", height);

    // Read video file into memory
    let buffer_size = width*height*3;
    let frames = read_video(in_path, buffer_size);
    println!("number of frames: {}", frames.len());
    println!();

    let raw_size = video_size(frames.len(), buffer_size);
    println!("(original rgb) size: {} MB\n", raw_size);

    let yuv_frames = yuv_encode(&frames, buffer_size, width, height);
    let yuv_size = video_size(yuv_frames.len(), yuv_frames.get(0).unwrap().len());
    println!("(converted yuv) size: {} MB", yuv_size);
    println!("(converted yuv) yuv/rgb: {} %\n", 100.0 * yuv_size / raw_size);

    // Write encoded video to file
    write_video(out_path, &yuv_frames);
}

// get video size in MB
fn video_size(len: usize, buffer_size: usize) -> f32 {
    (len * buffer_size) as f32 / MB
}

// TODO: look into using a Result<T> as the return instead
fn read_video(path: &str, buffer_size: usize) ->  Vec<Vec<u8>> {
    let mut frames = Vec::new();
    let file = fs::File::open(path).unwrap();
    let mut reader = BufReader::new(file);
    loop {
        let mut frame_buffer = vec![0; buffer_size];
        let n = reader.read(&mut frame_buffer).unwrap();
        if n < frames.len() {
            break;
        }

        frames.push(frame_buffer);
    }

    frames
}

// TODO: again, look into using Result<T> as return type
fn write_video(path: &str, frames: &Vec<Vec<u8>>) {
    let file = fs::File::options()
                        .write(true)
                        .create(true)
                        .open(path).unwrap();
    let mut writer = BufWriter::new(file);
    for frame in frames {
        writer.write(frame).unwrap();
    }
}

// TODO: this can probably be improved once a learn a bit more about rust
fn yuv_encode(frames: &Vec<Vec<u8>>, buffer_size: usize, width: usize, height: usize) -> Vec<Vec<u8>>{
    let mut yuv_frames = Vec::new();
    for frame in frames {
        let mut y: Vec<u8>  = vec![0; buffer_size/3];
        let mut u: Vec<f32> = vec![0.0; buffer_size/3];
        let mut v: Vec<f32> = vec![0.0; buffer_size/3];

        for n in 0..y.len() {
            let r = *frame.get(3*n).unwrap() as f32;
            let g = *frame.get(3*n+1).unwrap() as f32;
            let b = *frame.get(3*n+2).unwrap() as f32;

            let y_ =  0.299*r + 0.587*g + 0.114*b;
            let u_ = -0.169*r - 0.331*g + 0.449*b  + 128.0;
            let v_ =  0.499*r - 0.418*g - 0.0813*b + 128.0;

            y[n] = y_ as u8;
            u[n] = u_;
            v[n] = v_;
        }

        let mut u_downsampled = vec![0; buffer_size/3/4];
        let mut v_downsampled = vec![0; buffer_size/3/4];
        for x in (0..height).step_by(2) {
            for y in (0..width).step_by(2) {
                let u_ = (u.get(x*width+y).unwrap() + u.get(x*width+y+1).unwrap() + u.get((x+1)*width+y).unwrap() + u.get((x+1)*width+y+1).unwrap()) / 4.0;
                let v_ = (v.get(x*width+y).unwrap() + v.get(x*width+y+1).unwrap() + v.get((x+1)*width+y).unwrap() + v.get((x+1)*width+y+1).unwrap()) / 4.0;

                u_downsampled[x/2*width/2+y/2] = u_ as u8;
                v_downsampled[x/2*width/2+y/2] = v_ as u8;
            }
        }

        let mut yuv_frame_buffer  = Vec::new();
        yuv_frame_buffer.append(&mut y);
        yuv_frame_buffer.append(&mut u_downsampled);
        yuv_frame_buffer.append(&mut v_downsampled);

        yuv_frames.push(yuv_frame_buffer);
    }

    yuv_frames
}
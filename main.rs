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

    println!("(original rgb) size: {} MB", video_size(frames.len(), buffer_size));

    // TODO: actually write encoder and write it to new file
    // Write encoded video to file
    write_video(out_path, &frames);
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
                        .append(true)
                        .create(true)
                        .open(path).unwrap();
    let mut writer = BufWriter::new(file);
    for frame in frames {
        writer.write(frame).unwrap();
    }
}

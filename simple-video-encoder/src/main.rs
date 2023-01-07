use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};


const MB: f32 = 1e6;

type Frame = Vec<u8>;

#[derive(Parser)]
struct Args {
    // Video file to encode
    #[arg(short, default_value_t = String::from("video.rgb24"))]
    input_file: String,

    // Output directory
    #[arg(short, default_value_t = String::from("data/"))]
    output_dir: String,

    // Video width
    #[arg(long, default_value_t = 384)]
    width: usize,

    // Video height
    #[arg(long, default_value_t = 216)]
    height: usize,

    // Decode mode
    #[arg[short, default_value_t = false]]
    decode: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let in_path = args.input_file;
    let out_dir = args.output_dir;
    let width = args.width;
    let height = args.height;
    let decode = args.decode;

    let frame_size = width*height*3;

    if !decode {
        let yuv_out_path = [&out_dir, "/encoded.yuv"].concat();
        let rle_out_path = [&out_dir, "/encoded.rle"].concat();
        
        // Read video file into memory
        println!("reading in file...");
        let frames = match read_video(&in_path, frame_size) {
            Ok(vec) => {
                println!("finished reading in file");
                vec
            },
            Err(error) => panic!("Error reading file '{in_path}': {error}"),
        };
        let raw_size = video_size(&frames);
        println!("original size: {} MB\n", raw_size);

        // Convert from RGB to YUV and downsample
        println!("started yuv encoding...");
        let yuv_frames = rgb_to_yuv(&frames, width, height);
        println!("finished yuv encoding");
        let yuv_size = video_size(&yuv_frames);
        println!("new size: {} MB", yuv_size);
        println!("{}% of original size", 100.0 * yuv_size / raw_size);
        // Write yuv video to file
        println!("writing out file...");
        match write_file(&yuv_out_path, &yuv_frames) {
            Ok(_) => println!("finished writing out file\n"),
            Err(error) => panic!("Error writing file '{yuv_out_path}': {error}"),
        };
        

        // Encode file using RLE
        println!("started rle encoding...");
        let rle_frames_enc = rle_encode(&yuv_frames);
        println!("finished rle encoding");
        let rle_size = video_size(&rle_frames_enc);
        println!("new size: {} MB", rle_size);
        println!("{}% of original size", 100.0 * rle_size / raw_size);
        // write encoded video to file
        println!("writing out file...");
        match write_file(&rle_out_path, &rle_frames_enc) {
            Ok(_) => println!("finished writing out file\n"),
            Err(error) => panic!("Error writing file '{rle_out_path}': {error}"),
        };
        
    } else {
        // Read in encoded file
        let rle_out_path = [&out_dir, "/decoded.rle"].concat();
        println!("reading in file...");
        let rle_frames = match read_encoded(&in_path, frame_size/2) {
            Ok(vec) => {
                println!("finished reading in file");
                vec
            },
            Err(error) => panic!("Error reading file '{in_path}': {error}")
        };

        // Decode file
        println!("started file decoding...");
        let decoded_frames = rle_decode(&rle_frames, frame_size/2);
        println!("finished file decoding");
        println!("writing out file...");
        match write_file(&rle_out_path, &decoded_frames) {
            Ok(_) => println!("finished writing out file"),
            Err(error) => panic!("Error writing file '{rle_out_path}': {error}")
        };
    }

    Ok(())
}

// get video size in MB
fn video_size(frames: &Vec<Frame>) -> f32 {
    let mut size = 0;
    for frame in frames {
        size += frame.len();
    }

    size as f32 / MB
}

// read video into memory from input file
fn read_video(path: &str, size: usize) ->  io::Result<Vec<Frame>> {
    let mut frames = Vec::new();
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    loop {
        let mut frame= vec![0; size];
        let n = reader.read(&mut frame)?;
        if n < size {
            break;
        }
        frames.push(frame);
    }

    Ok(frames)
}

// read in encoded input video file
fn read_encoded(path: &str, size: usize) -> io::Result<Vec<Frame>> {
    let mut frames = Vec::new();
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut frame = Vec::with_capacity(size);
    let mut total = 0;
    for (i, result) in reader.bytes().enumerate() {
        let byte = result?;

        // first frame is stored unencoded
        if i < size {
            total += 1;
        } else if i % 2 == 0 {
            total += byte as usize;
        } 
        frame.push(byte);
        
        if total == size && (i+1) % 2 == 0 {
            frames.push(frame.to_vec());
            frame.clear();
            total = 0;
        }
    }

    Ok(frames)
}

// write to output file
fn write_file(path: &str, frames: &[Frame]) -> io::Result<()> {
    let file = fs::File::options()
                        .write(true)
                        .create(true)
                        .open(path)?;
    let mut writer = io::BufWriter::new(file);
    writer.write_all(&frames.concat())
}

// convert rgb to yuv
fn rgb_to_yuv(frames: &Vec<Frame>, width: usize, height: usize) -> Vec<Frame>{
    let size = width*height;

    let mut yuv_frames = Vec::with_capacity(frames.len());
    let mut y  = Vec::with_capacity(size);
    let mut u = Vec::with_capacity(size);
    let mut v = Vec::with_capacity(size);
    let mut u_avg= Vec::with_capacity(size/4);
    let mut v_avg = Vec::with_capacity(size/4);

    for frame in frames {
        // calculate y, u, v values
        for chunk in frame.chunks(3) {
            let r = chunk[0] as f32;
            let g = chunk[1] as f32;
            let b = chunk[2] as f32;

            y.push((0.299*r + 0.587*g + 0.114*b) as u8);
            u.push(-0.169*r - 0.331*g + 0.449*b  + 128.0);
            v.push(0.499*r - 0.418*g - 0.0813*b + 128.0);
        }

        // average pixels together
        let w = width;
        for i in (0..height).step_by(2) {
            for j in (0..width).step_by(2) {
                let u_ = (u[i*w+j] + u[i*w+j+1] + u[(i+1)*w+j] + u[(i+1)*w+j+1]) / 4.0;
                let v_ = (v[i*w+j] + v[i*w+j+1] + v[(i+1)*w+j] + v[(i+1)*w+j+1]) / 4.0;

                u_avg.push(u_ as u8);
                v_avg.push(v_ as u8);
            }
        }

        let mut yuv_frame= Vec::with_capacity(3/2*size);
        yuv_frame.append(&mut y);
        yuv_frame.append(&mut u_avg);
        yuv_frame.append(&mut v_avg);
        yuv_frames.push(yuv_frame);
        y.clear(); v.clear(); u.clear();
        u_avg.clear(); v_avg.clear();
    }

    yuv_frames
}

// perform run length encoding
fn rle_encode(frames: &Vec<Frame>) -> Vec<Frame> {
    let mut rle_frames = Vec::with_capacity(frames.len());
    for i in 0..frames.len() {
        if i == 0 {
            rle_frames.push(frames[i].to_vec());
            continue;
        }

        let size = frames[i].len();

        let mut rle = Vec::with_capacity(size*2);
        let mut prev = frames[i][0].wrapping_sub(frames[i-1][0]);
        let mut running_count = 1;
        for j in 1..size {
            let diff = frames[i][j].wrapping_sub(frames[i-1][j]);
            if running_count < 255 && diff == prev {
                running_count += 1;
            } else {
                rle.push(running_count as u8);
                rle.push(prev);
                running_count = 1;
                prev = diff;
            }

            // make sure to push last element
            if j == size - 1 {
                rle.push(running_count as u8);
                rle.push(diff);
            }
        }

        rle_frames.push(rle);
    }

    rle_frames
}

// decode rle encoded frames
fn rle_decode(frames: &Vec<Frame>, size: usize) -> Vec<Frame> {
    let mut rle_frames = Vec::with_capacity(frames.len());
    let mut delta = Vec::with_capacity(size);
    for i in 0..frames.len() {
        if i == 0 {
            rle_frames.push(frames[0].to_vec());
            continue;
        }

        for j in (0..frames[i].len()).step_by(2) {
            let count = frames[i][j];
            for _ in 0..count {
                delta.push(frames[i][j+1]);
            }
        }
        assert_eq!(size, delta.len());

        let mut decoded_frame = Vec::with_capacity(size);
        for j in 0..delta.len() {
            decoded_frame.push(delta[j].wrapping_add(rle_frames[i-1][j]));
        }
        delta.clear();
            
        rle_frames.push(decoded_frame);
    }

    rle_frames
}
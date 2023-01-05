use clap::Parser;

use std::fs;
use std::io;
use std::io::{Read, BufReader};
use std::io::{Write, BufWriter};


const MB: f32 = 1_000_000.;

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


struct ColorRGB{r: f32, g: f32, b: f32}
struct ColorYUV{y: f32, u: f32, v: f32}

impl ColorRGB {
    fn to_yuv(&self) -> ColorYUV {
        let y =  0.299*self.r + 0.587*self.g + 0.114*self.b;
        let u = -0.169*self.r - 0.331*self.g + 0.449*self.b  + 128.0;
        let v =  0.499*self.r - 0.418*self.g - 0.0813*self.b + 128.0;
        
        ColorYUV{y, u, v}
    }
}


// TODO: currently just lazily using unwrap, so need to
//      properly handle the Result<> later
fn main() -> io::Result<()> {
    let args = Args::parse();
    let in_path = args.input_file;
    let out_dir = args.output_dir;
    let width = args.width;
    let height = args.height;
    let decode = args.decode;

    if !decode {
        let yuv_out_path = [&out_dir, "/encoded.yuv"].concat();
        let rle_out_path = [&out_dir, "/encoded.rle"].concat();
        
        // Read video file into memory
        let frame_size = width*height*3;
        println!("reading in file...");
        let frames = read_video(&in_path, frame_size)?;
        println!("finished reading in file");

        let raw_size = video_size(&frames);
        println!("original size: {} MB\n", raw_size);

        // convert to yuv and downsample
        println!("started yuv encoding...");
        let yuv_frames = yuv_encode(&frames, width, height);
        println!("finished yuv encoding");
        let yuv_size = video_size(&yuv_frames);
        println!("{}% of original size", 100.0 * yuv_size / raw_size);
        // Write yuv video to file
        println!("writing out file...");
        write_file(&yuv_out_path, &yuv_frames)?;
        println!("finished writing out file\n");


        // rle encoder
        println!("started rle encoding...");
        let rle_frames = rle_encode(&yuv_frames);
        println!("finished rle encoding");
        let rle_size = video_size(&rle_frames);
        println!("{}% of original size", 100.0 * rle_size / raw_size);
        // write encoded video to file
        println!("writing out file...");
        write_file(&rle_out_path, &rle_frames)?;
        println!("finished writing out file\n");
    }

    // TODO: implemenet the 'read_encoded' function and figure out what 'size' should be
    else {
        let rle_out_path = [&out_dir, "/decoded.rle"].concat();
        println!("reading in file...");
        let rle_frames = read_encoded(&rle_out_path)?;
        println!("finished reading in file");
        println!("started file decoding...");
        let decoded_frames = rle_decode(&rle_frames, 0);
        println!("finished file decoding");
        println!("writing out file...");
        write_file(&rle_out_path, &decoded_frames)?;
        println!("finished writing out file");

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
fn read_video(path: &str, buffer_size: usize) ->  io::Result<Vec<Frame>> {
    let mut frames = Vec::new();
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    loop {
        let mut frame_buffer = vec![0; buffer_size];
        let n = reader.read(&mut frame_buffer)?;
        if n < frames.len() {
            break;
        }

        frames.push(frame_buffer);
    }

    Ok(frames)
}

// read in encoded input video file
fn read_encoded(path: &str) -> io::Result<Vec<Frame>> {
    Ok(Vec::new())
}

// write to output file
fn write_file(path: &str, frames: &Vec<Frame>) -> io::Result<()> {
    let file = fs::File::options()
                        .write(true)
                        .create(true)
                        .open(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&frames.concat())
}


// convert single frame from rgb to yuv
fn frame_to_yuv(frame: &Frame, y_buf: &mut Frame, u_buf: &mut Vec<f32>, v_buf: &mut Vec<f32>, size: usize) {
    for n in 0..size {
        let r = frame[3*n] as f32;
        let g = frame[3*n+1] as f32;
        let b = frame[3*n+2] as f32;

        let ColorYUV{y, u, v} = ColorRGB{r, g, b}.to_yuv();

        y_buf.push(y as u8);
        u_buf.push(u);
        v_buf.push(v);
    }
}

// downsample u & v
fn uv_downsample(u_buf: &Vec<f32>, v_buf: &Vec<f32>, 
                    u_down_buf: &mut Frame, v_down_buf: &mut Frame, 
                    width: usize, height: usize) {
    for x in (0..height).step_by(2) {
        for y in (0..width).step_by(2) {
            let u = (u_buf[x*width+y] + u_buf[x*width+y+1] + u_buf[(x+1)*width+y] + u_buf[(x+1)*width+y+1]) / 4.0;
            let v = (v_buf[x*width+y] + v_buf[x*width+y+1] + v_buf[(x+1)*width+y] + v_buf[(x+1)*width+y+1]) / 4.0;

            u_down_buf[x/2*width/2+y/2] = u as u8;
            v_down_buf[x/2*width/2+y/2] = v as u8;
        }
    }
}

fn yuv_encode(frames: &Vec<Frame>, width: usize, height: usize) -> Vec<Frame>{
    let mut yuv_frames = Vec::new();

    for frame in frames {
        let mut y_buf: Frame    = Vec::with_capacity(width*height);
        let mut u_buf: Vec<f32> = Vec::with_capacity(width*height);
        let mut v_buf: Vec<f32> = Vec::with_capacity(width*height);

        // calculate y, u, v values
        let size = y_buf.capacity();
        frame_to_yuv(frame, &mut y_buf, &mut u_buf, &mut v_buf, size);

        // average pixels together
        let mut u_downsampled = vec![0; width*height/4];
        let mut v_downsampled = vec![0; width*height/4];
        uv_downsample(&u_buf, &v_buf, &mut u_downsampled, &mut v_downsampled, width, height);

        let mut yuv_frame_buffer  = Vec::with_capacity(frame.len());
        yuv_frame_buffer.append(&mut y_buf);
        yuv_frame_buffer.append(&mut u_downsampled);
        yuv_frame_buffer.append(&mut v_downsampled);

        yuv_frames.push(yuv_frame_buffer);
    }

    yuv_frames
}

// TODO: is there a better way to handle overflow?
// compute difference and handle overflow
fn pixel_diff(val_1: u8, val_2: u8) -> u8 {
    if val_2 > val_1 {
        return (256 - (val_2 as u16) + (val_1 as u16)) as u8
    }

    val_1 - val_2
}

// compute summation and handel overflow
fn pixel_sum(val_1: u8, val_2: u8) -> u8 {
    let sum = (val_1 as u16) + (val_2 as u16);
    if sum > 255 {
        return (sum - 256) as u8
    }

    sum as u8
}

fn rle_encode(frames: &Vec<Frame>) -> Vec<Frame> {
    let mut rle_frames = Vec::with_capacity(frames.len());
    for i in 0..frames.len() {
        if i == 0 {
            rle_frames.push(frames[i].to_vec());
            continue;
        }

        // get difference between each frame
        let mut delta = Vec::with_capacity(frames[i].len());
        for j in 0..delta.capacity() {
            delta.push(pixel_diff(frames[i][j], frames[i-1][j]));
        }

        // compute run length encoding on frame differences
        let mut rle = Vec::new();
        let mut j = 0;
        while j < delta.len() {
            let mut count = 0;
            while count < 255 && j+count < delta.len() && delta[j+count] == delta[j] {
                count += 1;
            }
            rle.push(count as u8);
            rle.push(delta[j]);

            j += count;
        }


        rle_frames.push(rle);
    }

    rle_frames
}

fn rle_decode(frames: &Vec<Frame>, size: usize) -> Vec<Frame> {
    let mut rle_frames = Vec::with_capacity(frames.len());
    for i in 0..frames.len() {
        if i == 0 {
            rle_frames.push(frames[0].to_vec());
            continue;
        }

        let mut delta = Vec::with_capacity(size);
        for j in (0..frames[i].len()).step_by(2) {
            let count = frames[i][j];
            for _ in 0..count {
                delta.push(frames[i][j+1]);
            }
        }
        assert_eq!(size, delta.len());

        let mut decoded_frame = Vec::with_capacity(size);
        for j in 0..delta.len() {
            decoded_frame.push(pixel_sum(rle_frames[i-1][j], delta[j]));
        }
            
        rle_frames.push(decoded_frame);
    }

    rle_frames
}
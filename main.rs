use std::env;
use std::fs;

use std::io;
use std::io::{Read, BufReader};
use std::io::{Write, BufWriter};

const MB: f32 = 1_000_000.;

type Frame = Vec<u8>;

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
// properly handle the Result<> later
fn main() -> io::Result<()> {
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
    let frames = read_video(in_path, buffer_size)?;
    println!("number of frames: {}", frames.len());
    println!();

    let raw_size = video_size(frames.len(), buffer_size);
    println!("(original rgb) size: {} MB\n", raw_size);

    // convert to yuv and downsample
    let yuv_frames = yuv_encode(&frames, width, height);
    let yuv_size = video_size(yuv_frames.len(), yuv_frames.get(0).unwrap().len());

    println!("(converted yuv) size: {} MB", yuv_size);
    println!("(converted yuv) yuv/rgb: {} %\n", 100.0 * yuv_size / raw_size);


    // Write encoded video to file
    // TODO: write rle encoder
    write_video(out_path, &yuv_frames)
}

// get video size in MB
fn video_size(len: usize, buffer_size: usize) -> f32 {
    (len * buffer_size) as f32 / MB
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

// write vidoe to output file
fn write_video(path: &str, frames: &Vec<Frame>) -> io::Result<()> {
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
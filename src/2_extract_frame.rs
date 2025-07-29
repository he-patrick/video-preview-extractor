use video_rs::decode::Decoder;
use video_rs::Url;
use image::{ImageBuffer, RgbImage};

fn main() {
    // Initialize video-rs (a wrapper for ffmpeg-next)
    video_rs::init().unwrap();

    // Take URL, parse it into a video_rs URL type
    let absolute_path = std::path::Path::new("./test_vid.mov")
        .canonicalize()
        .unwrap();
    let source = format!("file://{}", absolute_path.display())
        .parse::<Url>()
        .unwrap();

    // Create decoder
    let mut decoder = Decoder::new(source).expect("failed to create decoder");

    // Decode the first frame
    let (_, frame) = decoder.decode().expect("Failed to decode");

    // get the frame size
    let (width, height) = decoder.size();

    /*
    Converts from a 2D structure to a 1D array:
    [(r,g,b), (r,g,b), (r,g,b), ...] => [r,g,b,r,g,b,r,g,b,...]
    */
    let (raw, _) = frame.into_raw_vec_and_offset();

    let img: RgbImage = ImageBuffer::from_raw(width, height, raw).expect("Failed to create image");
    img.save("first_frame.png").expect("Failed to save frame");
}
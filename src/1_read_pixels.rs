use video_rs::decode::Decoder;
use video_rs::Url;

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

    for frame in decoder.decode_iter() {

        if let Ok((_, frame)) = frame {
            let rgb = frame.slice(ndarray::s![0,0,..]).to_slice().unwrap();
            println!("pixels at 0, 0: {}, {}, {}", rgb[0], rgb[1], rgb[2],);
        }
        else {
            break;
        }
    }

}
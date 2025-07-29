use ffmpeg_next as ffmpeg;
use ffmpeg_next::{format, software, util};
use ffmpeg_next::util::mathematics::{rescale, Rescale};
use image::{ImageBuffer, RgbaImage};

fn main() {
    // Initialize ffmpeg-next wrapper
    ffmpeg::init().unwrap();

    // Take URL, parse it into a video_rs URL type
    let input = "./test_vid.mov";
    let seek = [0.0, 0.25, 0.5, 0.75];
    let mut frames = Vec::new();

    let mut ictx = format::input(&input).expect("Failed to open input file");
    let input_stream = ictx.streams().best(ffmpeg::media::Type::Video).expect("Failed to find best video stream");
    let video_stream_index = input_stream.index();
    
    // Get the duration of the video, in time base units
    let duration = input_stream.duration();

    // Get the time base
    let time_base = input_stream.time_base();
    
    // Convert duration to seconds
    let duration_seconds = duration as f64 * time_base.0 as f64 / time_base.1 as f64;
    
    println!("Video duration: {} time units, {} seconds", duration, duration_seconds);

    // Get video parameters for decoder creation
    let video_params = input_stream.parameters();

    // Get the video stream index
    let stream_index = video_stream_index;

    // Create a single decoder context outside the loop
    let context_decoder = ffmpeg::codec::context::Context::from_parameters(video_params.clone()).expect("Failed to create context");
    let mut decoder = context_decoder.decoder().video().expect("Failed to create video decoder");

    // Create a scaler to convert to RGBA format
    let mut scaler = software::scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        util::format::Pixel::RGBA,
        decoder.width(),
        decoder.height(),
        software::scaling::Flags::BILINEAR,
    ).expect("Failed to create scaler");

    for &s in &seek {
        // Convert seek position to seconds, then rescale to FFmpeg's base timebase
        let target_seconds = (duration_seconds * s as f64) as i64;
        let target_timestamp = target_seconds.rescale((1, 1), rescale::TIME_BASE);

        println!("Seeking to {}% ({} seconds, rescaled timestamp: {})", s * 100.0, target_seconds, target_timestamp);
        
        // Seek to the target timestamp
        let seek_result = ictx.seek(target_timestamp, ..target_timestamp);
        
        match seek_result {
            Ok(_) => println!("Seek successful to timestamp {}", target_timestamp),
            Err(e) => {
                println!("Seek failed: {:?}", e);
                continue;
            }
        }

        // Flush the decoder after seeking
        decoder.flush();

        // Read a few packets after the seek to get a frame
        let mut packets_processed = 0;
        let max_packets = 20; // Only process a few packets after seeking

        for (stream, packet) in ictx.packets() {
            if stream.index() != stream_index {
                continue;
            }

            packets_processed += 1;
            if packets_processed > max_packets {
                break;
            }
            
            decoder.send_packet(&packet).expect("Failed to send packet to decoder");

            let mut frame = util::frame::Video::empty();
            if decoder.receive_frame(&mut frame).is_ok() {
                let frame_pts = frame.timestamp();
                if let Some(pts) = frame_pts {
                    println!("Got frame at timestamp: {} for seek target: {}", pts, target_timestamp);
                }
                
                // Convert the frame to RGBA format
                let mut rgba_frame = util::frame::Video::empty();
                scaler.run(&frame, &mut rgba_frame).expect("Failed to scale frame");

                // Grab raw pixel data and properties
                let data = rgba_frame.data(0);
                let linesize = rgba_frame.stride(0);
                let height = rgba_frame.height() as usize;
                let width = rgba_frame.width() as usize;

                let mut rgba = Vec::with_capacity(width * height * 4);

                // Create flat RGBA buffer
                for y in 0..height {
                    let row = &data[y * linesize..y * linesize + width * 4];
                    rgba.extend_from_slice(row);
                }

                frames.push((width as u32, height as u32, rgba));
                println!("Extracted frame {} from timestamp {:?}", frames.len() - 1, frame_pts);
                break;
            }
        }
    }

    // Save and display the extracted frames
    println!("Extracted {} frames", frames.len());
    
    for (i, (width, height, rgba_data)) in frames.iter().enumerate() {
        // Create an image buffer from the RGBA data
        let img: RgbaImage = ImageBuffer::from_raw(*width, *height, rgba_data.clone())
            .expect("Failed to create image from raw data");
        
        // Convert RGBA to RGB for saving as JPEG
        let rgb_img = image::DynamicImage::ImageRgba8(img).to_rgb8();
        
        // Save the frame as an image file
        let filename = format!("frame_{:02}.jpg", i);
        rgb_img.save(&filename).expect("Failed to save image");
        
        println!("Saved frame {} as {} ({}x{})", i, filename, width, height);
    }
}
#![deny(missing_docs)]

//! # hilbert_image_to_sound
//! A library for turning images into sound using Hilbert space-filling curves
//!
//! [Links to other related research projects](https://github.com/advancedresearch/hilbert_image_to_sound/issues/3)
//!
//! *Notice: This library is in rough shape now,
//! just to demonstrating the concept. PRs are welcome!*

/// Plays an image file as sound.
pub fn play(file: &str) {
    use cpal::traits::{DeviceTrait, HostTrait, EventLoopTrait};

    let image = image::open(file).unwrap().to_rgba();

    // Controls the resolution of transformed image dimensions.
    let n = 16;
    // Stores the Hilbert curve 1d coordinates for each pixel position.
    let mut hilbert: Vec<Vec<usize>> = vec![vec![0; n]; n];
    // Create an inverse map.
    let n2 = n as usize * n as usize;
    for i in 0..n2 {
        let (x, y) = hilbert_curve::convert_1d_to_2d(i, n2);
        hilbert[y][x] = i;
    }

    // Stores sound frequency amplitudes.
    let mut amplitudes: Vec<f64> = vec![0.0; n2];
    let (w, h) = image.dimensions();
    let cell = w.min(h) / n as u32;
    for (x, y, pixel) in image.enumerate_pixels() {
        let x = x / cell;
        let y = y / cell;
        if x >= n as u32 || y >= n as u32 {continue}
        let image::Rgba([r, g, b, _]) = pixel;
        let frequency = hilbert[y as usize][x as usize];
        let amplitude = (*r as f64 + *g as f64 + *b as f64) / 255.0 / 3.0;
        amplitudes[frequency] += amplitude / (cell as f64 * cell as f64);
    }

    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let device = host.default_output_device().expect("no output device available");

    let mut supported_formats_range = device.supported_output_formats()
        .expect("error while querying formats");
    let format = supported_formats_range.next()
        .expect("no supported format?!")
        .with_max_sample_rate();

    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id).expect("failed to play_stream");

    let tau: f64 = 6.283185307179586;
    let mut t: f64 = 0.0;
    let volume = 0.2;
    std::thread::spawn(move || {

        event_loop.run(move |stream_id, stream_result| {
            use cpal::{StreamData, UnknownTypeOutputBuffer};

            let stream_data = match stream_result {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                    return;
                }
            };

            match stream_data {
                StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
                    for elem in buffer.iter_mut() {
                        let mut s: f64 = 0.0;
                        for (i, amp) in amplitudes.iter().enumerate() {
                            let fi = i as f64 / n2 as f64;
                            let f = 4.5 * fi + 0.25;
                            s += *amp * (t * f * tau + i as f64).sin();
                        }
                        *elem = volume * s as f32;

                        t += 0.005;
                    }
                },
                _ => (),
            }
        });
    });

    std::thread::sleep(std::time::Duration::from_secs_f64(2.0))
}

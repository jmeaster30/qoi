mod qoi;

use pixel_canvas::{Canvas, Color, input::MouseState};
use glob::glob;
use std::cmp;
use std::io;
use std::time::Instant;

fn first_mismatch(a: Vec<u8>, b: Vec<u8>) -> usize
{
    let longest = if a.len() > b.len() { a.len() } else { b.len() };
    for i in 0..longest {
        if i >= a.len() || i >= b.len() {
            return i;
        }
        let av = a[i];
        let bv = b[i];
        if av != bv {
            return i;
        }
    }
    return longest;
}

fn main() -> io::Result<()>{

    let command = std::env::args().nth(1).expect("Missing command. Expected 'test', 'encode', or 'decode'");

    match command.as_str() {
        "test" => {
            let mut pass = 0; 
            let mut total = 0;

            for glob_entry in glob("qoi_test_images/testcard.qoi").unwrap() {
                match glob_entry {
                    Ok(file) => {
                        let filename = &file.into_os_string().into_string().unwrap();

                        println!("\nLoading file {}", filename);

                        let qfile = qoi::QoiFile::load_from_file(filename)?;
                        let initial_encoded = qfile.encoded.clone();

                        let now = Instant::now();
                        let decoded = qfile.decode();
                        let elapsed = now.elapsed();

                        //let w = decoded.width as usize;
                        //let h = decoded.height as usize;

                        println!("Finished decoding image '{}'\tElapsed: {:.2?}", filename, elapsed);
                        //println!("Decoded Length {} - W {} x H {}", decoded.decoded.len(), w, h);

                        let now = Instant::now();
                        let final_encoded = decoded.encode();
                        let elapsed = now.elapsed();

                        println!("Finished encoding image '{}'\tElapsed: {:.2?}", filename, elapsed);

                        if initial_encoded == final_encoded.encoded {
                            println!("{} - PASS", filename);
                            pass += 1;
                        } else {
                            println!("{} - FAILED", filename);
                        }

                        let distance = 10;
                        let first_fail_idx = first_mismatch(initial_encoded.clone(), final_encoded.encoded.clone());
                        let i_min = 0;//first_fail_idx - distance;
                        let i_max = cmp::min(first_fail_idx + distance, initial_encoded.len());
                        let f_min = 0;//first_fail_idx - distance;
                        let f_max = cmp::min(first_fail_idx + distance, final_encoded.encoded.len());
                        println!("{}", first_fail_idx);                        
                        println!("{} {}", i_min, i_max);
                        println!("{} {}", f_min, f_max);

                        println!("{:?}", &(initial_encoded[i_min..i_max]));
                        println!("{:?}", &(final_encoded.encoded[f_min..f_max]));

                        total += 1;
                    },
                    Err(e) => println!("{:?}", e)
                }
                
            }

            println!("{} out of {} pics passed.", pass, total);
        },
        "decode" => {
            let filename = std::env::args().nth(2).expect("Missing filename to decode");
            let qoifile = qoi::QoiFile::load_from_file(&filename)?;
            
            let now = Instant::now();
            let decoded = qoifile.decode();
            let elapsed = now.elapsed();

            let w = decoded.width as usize;
            let h = decoded.height as usize;

            println!("Finished decoding image '{}'\tElapsed: {:.2?}", filename, elapsed);

            let canvas = Canvas::new(w, h)
                .title(filename)
                .state(MouseState::new())
                .input(MouseState::handle_input);

            canvas.render(move |_mouse, image| {
                for (y, row) in image.chunks_mut(w).enumerate() {
                    for (x, pixel) in row.iter_mut().enumerate() {
                        *pixel = Color {
                            r: decoded.get(x, h - y).red,
                            g: decoded.get(x, h - y).green,
                            b: decoded.get(x, h - y).blue,
                        }
                    }
                }
            });
        }
        "encode" => {
            println!("encode");
        },
        _ => println!("Unknown command {}. Expected 'test', 'encode', or 'decode'", command)
    }

    Ok(())
}

use scrap::{Capturer, Display};
use std::io::ErrorKind;
use std::is_x86_feature_detected;
use std::sync::{Arc, RwLock};

mod converters;
//use converters::{ avx2_cmp_and_convert, avx2_bgra_to_rgba, dlwbitop_bgra_to_rgba};
use converters::{ avx2_bgra_to_rgba };

pub struct Snapper {
    capturer: Capturer,
    pub width: usize,
    pub height: usize,
}
unsafe impl Send for Snapper {}
unsafe impl Sync for Snapper {}

impl Drop for Snapper {
    fn drop(&mut self) {
    }
}

impl Snapper {
    pub fn new() -> Self {
        if ! is_x86_feature_detected!("avx2") {
            panic!("This library depends upon the AVX2 extensions, which this processor does not support");
        }

        let display = Display::primary().expect("Couldn't get a handle for the primary display");
        let capturer = Capturer::new(display).expect("Couldn't instatiate capturer for the primary display");
        let width = capturer.width();
        let height = capturer.height();

        Snapper{
            capturer,
            width,
            height,
        }
    }

    pub fn snap(&mut self) -> Vec<u8> {
        loop {
            match self.capturer.frame() {
                Ok(buf) => {
                    let stride = ( buf.len() / self.height ) / 4; // may include padding.
                    let mut bgra = buf.to_vec();
                    let mut rgba = vec![0; buf.len()];
                    unsafe {
                        for i in (0 .. buf.len() ).step_by(32) {
                            avx2_bgra_to_rgba(i as isize, &bgra, &mut rgba);
                        }
                    }
                    /*
                    unsafe { // scrap actually blocks until the screen changes, so none of this helps.
                             // for a potential future replacement with "better" features it might
                             // matter though.
                        for i in (0 .. buf.len() ).step_by(32) {
                            let diff = avx2_cmp_and_convert(i as isize, &bgra, &mut self.last);
                            if diff == true { changed = true; }
                        }
                        if changed == true { 
                            break; 
                        } else { 
                            self.last = bgra;
                            continue;
                        }
                    }
                    */
                    return rgba;
                },
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => continue,
                    _ => panic!("Screen grab failed with error {:?}", e),
                },
            }
        }
    }
}

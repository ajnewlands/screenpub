use scrap::{Capturer, Display};
use std::io::ErrorKind;
use std::is_x86_feature_detected;
use log::debug;

mod converters;
use converters::{ avx2_bgra_to_rgba, avx2_cmp_and_convert, avx2_cmp, avx2_convert_in_place };

#[derive(PartialEq, Eq, Debug)]
enum RegionType {
    Unknown = 0,
    Similarity = 1,
    Difference = 2,
}

pub struct Bigtile {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    pub tile: Vec::<u8>,
}

impl Bigtile {
    // TODO this will be shit slow.
    pub fn from_image(image: &Vec<u8>, bpr: i32, startx: i32, starty: i32, endx: i32, endy: i32) -> Self {
        let x = (startx/4) as u16; // count in pixels, not bytes, for front end.
        let y = starty as u16;
        let w = ((endx - startx) / 4) as u16;
        let h = (endy - starty) as u16;
        let mut tile = Vec::<u8>::with_capacity( ((endx-startx) * (endy-starty)) as usize );

        let now = std::time::Instant::now();
        for j in starty..endy {
            let mut v = Vec::with_capacity( (endx-startx) as usize );
            unsafe{ v.set_len( (endx-startx) as usize);}
            v.clone_from_slice( &image[ ((j*bpr + startx) as usize)..((j*bpr  + endx) as usize)]);
            tile.append(&mut v);
        }
        debug!("tile copy took {} ms", now.elapsed().as_millis());
        Bigtile{x, y, w, h, tile}
    }
}

pub struct Snapper {
    capturer: Capturer,
    pub width: usize,
    pub height: usize,
    pub last: Vec<u8>, // a copy of the previous screen buffer for incremental modes.
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
            last: Vec::with_capacity(0), //don't attempt to presize because stride != width.
        }
    }

    fn get_frame(&mut self) -> Vec<u8> {
        loop {
            match self.capturer.frame() {
                Ok(buf) => return buf.to_vec(),
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => continue,
                    _ => panic!( "Could not capture screen: {:?}", e),
                },
            };
        }
    }

    pub fn snap_bigtiles(&mut self) -> Vec<Bigtile> {
        let mut tiles = Vec::<Bigtile>::new();
        let mut pixels = self.get_frame();       
        let bytes_per_row = pixels.len() / self.height;

        if self.last.len() != pixels.len() {
            panic!("Called for a hextile update but the previous framebuffer length {} does not match current {}", self.last.len(), pixels.len());
        }

        // convert the pixels to RGBA before comparing against the previous frame (which is RGBA)
        unsafe { 
            for i in (0 .. pixels.len() ).step_by(32) {
                avx2_convert_in_place(i as isize, &mut pixels);
            }
        }

        let now = std::time::Instant::now();
        // start splitting the regions horizontally
        //tiles.append( &mut self.split_h(&pixels, bytes_per_row as i32, 0, 0, bytes_per_row as i32, self.height as i32) );
        tiles.append( &mut self.son_of_krunch(&pixels, bytes_per_row as i32, 0, 0, bytes_per_row as i32, self.height as i32) );
        println!("done krunching");
        self.last = pixels.clone();
        debug!("tiling took {} ms", now.elapsed().as_millis());

        tiles
    }

    // attempt at an O(n) split algorithm
    fn son_of_krunch(&mut self, data: &Vec<u8>, bytes_per_row: i32, startx: i32, starty: i32, endx: i32, endy: i32) -> Vec<Bigtile> {
        println!("Invoked with sx {}, sy {}, ex {}, ey {}", startx, starty, endx, endy);
        let mut tiles = Vec::<Bigtile>::new();
        let mut region_type = RegionType::Unknown;

        for y in (starty..endy).step_by(16) { // define a 16px x 16px (or less at the edges) tile.
            for x in (startx .. endx).step_by(64) { // 16 px X 4 channel RGBA
                let mut identical = true; // start by assuming this tile is identical between screens
                for row in 0..16 {
                    unsafe { 
                        let left = (( (y + row) * bytes_per_row) + x) as isize;
                        identical = avx2_cmp( left, &data, &mut self.last ) && identical; 
                        let right = (( (y + row) * bytes_per_row) + x + 32) as isize;
                        identical = avx2_cmp( right, &data, &mut self.last ) && identical; 
                    }
                }
                // the first tile determines whether we will extract a region of similarity or
                // difference.
                if region_type == RegionType::Unknown{
                    region_type = match identical {
                        true => RegionType::Similarity,
                        false => RegionType::Difference,
                    };
                } else if (region_type == RegionType::Difference) && identical {
                    println!("Run of differences ended @ {}, {} when searching {}, {} to {}, {}", x, y, startx, starty, endx, endy);
                    // End of a region of difference: send everything above the current row
                    if(y-16 > starty) {
                        println!("bigtiling sx {}, sy {}, ex {}, ey {}", startx, starty, endx, y-16);
                        tiles.push( Bigtile::from_image(&data, bytes_per_row, startx, starty, endx, y-16));
                    }

                    // (2) search the region to the right from the current row
                    //println!("right side will be {}, {} to {}, {}", x, y, endx, endy);
                    tiles.append( &mut self.son_of_krunch(data, bytes_per_row, x, y, endx, endy));

                    // (3) search the region to the left on the current row
                    if x-64 > startx {
                        //println!("left side will be {}, {} to {}, {}", startx, y, x-64, endy);
                        tiles.append( &mut self.son_of_krunch(data, bytes_per_row, startx, y, x, endy)); // should this actuall end at x+64?
                    }
                    //
                    return tiles;
                } else if (region_type == RegionType::Similarity) && !identical {
                    // End of a region of similarity: (1) discard everything above the current
                    // row which is unchanged
                    // (2) search the region to the left below the current row (as everything
                    // immediately left is unchanged)
                    if x > startx {
                        tiles.append( &mut self.son_of_krunch(data, bytes_per_row, startx, y + 16, x, endy));
                    }
                    // (3) search the region to the right on the current row
                    tiles.append( &mut self.son_of_krunch(data, bytes_per_row, x, y, endx, endy));
                    return tiles;
                } else if region_type == RegionType::Difference && x == endx && y == endy {
                    tiles.push( Bigtile::from_image(&data, bytes_per_row, startx, starty, endx, endy));
                    return tiles;
                }
            }
        }

        tiles
    }

    fn split_v(&mut self, data: &Vec<u8>, bytes_per_row: i32, startx: i32, starty: i32, endx: i32, endy: i32) -> Vec<Bigtile> {
        let mut tiles = Vec::<Bigtile>::new();

        let mut sx: i32 = endx;
        let mut ex: i32 =  -1;

        for x in (startx .. endx).step_by(64) { // 16 px X 4 channel RGBA
            let mut identical = true;

            for y in (starty..endy).step_by(16) { // define a 16px x 16px (or less at the edges) tile.
                for row in 0..16 {
                    unsafe { 
                        let left = (( (y + row) * bytes_per_row) + x) as isize;
                        identical = avx2_cmp( left, &data, &mut self.last ) && identical; 
                        let right = (( (y + row) * bytes_per_row) + x + 32) as isize;
                        identical = avx2_cmp( right, &data, &mut self.last ) && identical; 
                    }
                }
            }

            if ! identical {
                if x < sx { sx = x; }
                if x + 64 > ex { ex = x + 64};
            }
            if (identical && ex != -1) || (!identical && (x + 64) >= endx) {
                // if we didn't split the area, return one big tile
                if( sx == startx && ex == endx ) {
                    tiles.push( Bigtile::from_image(&data, bytes_per_row, startx, starty, endx, endy));
                } else {  // keep trying to split until we can't (consists of one 16px sq)
                    tiles.append( &mut self.split_h(data, bytes_per_row, sx, starty, ex, endy) );
                }
                sx = endx;
                ex = -1;
            }
        }

        tiles
    }

    fn split_h(&mut self, data: &Vec<u8>, bytes_per_row: i32, startx: i32, starty: i32, endx: i32, endy: i32) -> Vec<Bigtile> {
        let mut tiles = Vec::<Bigtile>::new();

        let mut sy: i32 = endy;
        let mut ey: i32 =  -1;

        for y in (starty..endy).step_by(16) { // define a 16px x 16px (or less at the edges) tile.
            let mut identical = true;

            // test whether this row of tiles differs from the previous frame
            for x in (startx .. endx).step_by(64) { // 16 px X 4 channel RGBA
                for row in 0..16 {
                    unsafe { 
                        let left = (( (y + row) * bytes_per_row) + x) as isize;
                        identical = avx2_cmp( left, &data, &mut self.last ) && identical; 
                        let right = (( (y + row) * bytes_per_row) + x + 32) as isize;
                        identical = avx2_cmp( right, &data, &mut self.last ) && identical; 
                    }
                }
            }

            if ! identical {
                if y < sy { sy = y; }
                if y + 16 > ey { ey = y + 16};
            }
            if (identical && ey != -1) || (!identical && (y+16) >= endy) { 
                // if we didn't split the area, return one big tile
                if( sy == starty && ey == endy ) {
                    tiles.push( Bigtile::from_image(&data, bytes_per_row, startx, starty, endx, endy));
                } else {  // keep trying to split until we can't (consists of one 16px sq)
                    tiles.append( &mut self.split_v(data, bytes_per_row, startx, sy, endx, ey) );
                }
                sy = endy;
                ey = -1;
            }
        }

        tiles
    }

    // Populate last, no matter what may have been there before
    pub fn snap(&mut self) -> Vec<u8> {
        loop {
            match self.capturer.frame() {
                Ok(buf) => {
                    let bgra = buf.to_vec();
                    self.last = Vec::with_capacity(buf.len());

                    unsafe {
                        self.last.set_len(buf.len());

                        for i in (0 .. buf.len() ).step_by(32) {
                            avx2_bgra_to_rgba(i as isize, &bgra, &mut self.last);
                        }
                    }
                    break;
                },
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => continue,
                    _ => panic!("Screen grab failed with error {:?}", e),
                },
            };
        }

        self.last.clone()
    }
}

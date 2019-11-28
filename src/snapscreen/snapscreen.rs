use scrap::{Capturer, Display};
use std::io::ErrorKind;
use std::is_x86_feature_detected;

mod converters;
use converters::{ avx2_bgra_to_rgba, avx2_cmp_and_convert, avx2_cmp, avx2_convert_in_place };

pub struct Hextile {
    pub x: u16,
    pub y: u16,
    pub tile: Vec::<u8>,
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

        for j in starty..endy {
            for i in startx..endx {
                tile.push( image[ (j*bpr + i) as usize] );
            }
        }
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

        // start splitting the regions horizontally
        tiles.append( &mut self.split_h(&pixels, bytes_per_row as i32, 0, 0, bytes_per_row as i32, self.height as i32) );
        self.last = pixels.clone();

        return tiles;
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

        return tiles;
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

        return tiles;
    }

    /// populate "last" and return the number of tiles that would be updated.
    pub fn snap_hextile(&mut self) -> Vec<Hextile> {
        let mut tiles = Vec::<Hextile>::new();
        let bgra = self.get_frame();       
        let bytes_per_row = bgra.len() / self.height;

        if self.last.len() != bgra.len() {
            panic!("Called for a hextile update but the previous framebuffer length {} does not match current {}", self.last.len(), bgra.len());
        }
        
        let mut tile = Vec::with_capacity( 16 * 16 * 4 ); // TODO CONST

        for x in (0..bytes_per_row).step_by(64) { // N.B. IA64 prefetch size is 64 bytes

            for y in (0..self.height).step_by(16) { // define a 16px x 16px (or less at the edges) tile.
                let mut unchanged = true;
                // test along the horizontal plane first to match the L1 prefetch.
                // AVX2 version doing 256 bits at a time
                for row in 0..16 {
                    unsafe { 
                        let left = (( (y + row) * bytes_per_row) + x) as isize;
                        unchanged = avx2_cmp_and_convert( left, &bgra, &mut self.last ) && unchanged; 
                        let right = (( (y + row) * bytes_per_row) + x + 32) as isize;
                        unchanged = avx2_cmp_and_convert( right, &bgra, &mut self.last ) && unchanged; 
                    }
                }
                if !unchanged {
                    for row in 0..16 {
                        let offset =  ((y + row) * bytes_per_row) + x;
                        tile.extend( &self.last[offset..(offset + 64)] );
                    }
                    tiles.push(Hextile{ x: (x/64) as u16, y: (y/16) as u16, tile: tile.clone() });
                    tile.clear();
                }
            }
        }
        // Windows will issue an "update" when the mouse moves, even if nothing in the actual
        // capture changed.
        return match tiles.len() {
            0 => self.snap_hextile(),
            _ => tiles,
        };
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
        return self.last.clone();
    }

    /*
    pub fn snap(&mut self, capabilities: u32) -> Vec<u8> {
        // generate an update according to the "best" mode possible.
        loop {
            match self.capturer.frame() {
                Ok(buf) => {
                    let stride = ( buf.len() / self.height ) / 4; // may include padding.
                    let mut bgra = buf.to_vec();
                    // if image sizes don't match, or no "higher" capabilities are available, a
                    // full screen update is implied.

                    // for now we fudge it; rely on the initial update having capability = 0 and
                    // dont check size at all.

                    // Initial screen update (or no incremental capability) or screen has changed
                    // dimensions (which I don't think scrap can handle anyway)
                    if self.last.len() == 0 || self.last.len() != bgra.len() { 
                        let mut rgba = Vec::with_capacity(buf.len());
                        unsafe {
                            rgba.set_len(buf.len());

                            for i in (0 .. buf.len() ).step_by(32) {
                                avx2_bgra_to_rgba(i as isize, &bgra, &mut rgba);
                            }
                        }

                        return match capabilities {
                            0 => rgba, // no need to retain state when doing only fullscreen updates
                            _ => {
                                self.last = rgba.clone();
                                rgba
                            },
                        }
                        
                    } else { // incremental update
                        unsafe {
                            for i in (0 .. buf.len() ).step_by(32) {
                                avx2_convert_with_deltas(i as isize, &mut bgra, &mut self.last);
                            }
                            return bgra;
                        }
                    }

                    // this is actually not true; if we have part of a window onscreen, and that
                    // window has focus, we will get a stream of unnecessary updates regardless.
                    //
                    // ideally, want a function that 
                    // (1) compares
                    // (2) converts
                    // (3) xors
                    // keeps one pristine copy of the original plus one set of deltas.
                    
                    /*
                    unsafe { 
                        rgba.set_len(buf.len()); // can we do this in-place, i.e. write the modified pixels to bgra instead of rgba?
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
                    /*
                    if self.last.len() != rgba.len() {
                        self.last.reserve_exact(rgba.len());
                        unsafe{ self.last.set_len( rgba.len() ); }
                    }

                    if (capabilities == 1) { // clobber rgba with (last xor rgba)
                        rgba.iter_mut().zip(self.last.iter()).for_each(|(a,b)| *a ^= *b); // compute the deltas for transmission.
                        self.last.iter_mut().zip(rgba.iter()).for_each(|(a,b)| *a ^= *b); // update last using the deltas
                    } else {
                        self.last = rgba.clone();
                    }

                    return rgba; // either a full update or bitwise deltas.
                    */

                },
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => continue,
                    _ => panic!("Screen grab failed with error {:?}", e),
                },
            }
        }
    }
    */
}

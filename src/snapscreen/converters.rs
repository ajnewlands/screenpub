use std::arch::x86_64::{ __m256i, _mm256_shuffle_epi8, __m128i, _mm_shuffle_epi8, _mm256_cmpeq_epi64, _mm256_movemask_epi8};

pub fn naive_bgra_to_rgba(offset: usize, bgra: &Vec<u8>, rgba: &mut Vec<u8>) {
    rgba[offset] = bgra[offset+2];
    rgba[offset+1] = bgra[offset+1];
    rgba[offset+2] = bgra[offset];
    rgba[offset+3] = bgra[offset+3];
}

pub unsafe fn dlwbitop_bgra_to_rgba( offset: isize, bgra: &Vec<u8>, rgba: &mut Vec<u8>) {
    let lw_bgra =  std::ptr::read( bgra.as_ptr().offset(offset) as *const u64 );
    let lw_rgba = (lw_bgra & 0xff00ff00ff00ff00) 
        | ( (lw_bgra & 0x000000ff000000ff) << 16) 
        | ( (lw_bgra & 0x00ff000000ff0000) >> 16);
    std::ptr::write( rgba.as_mut_ptr().offset(offset) as *mut u64, lw_rgba);
}

#[target_feature(enable="avx2")]
pub unsafe fn avx2_bgra_to_rgba(offset: isize, bgra: &Vec<u8>, rgba: &mut Vec<u8>) {
    const SHUFFLE: [i8; 32] = [2, 1, 0, 3,
                                6, 5, 4,7,
                                10, 9, 8, 11,
                                14, 13, 12, 15,
                                18, 17, 16, 19,
                                22, 21, 20, 23,
                                26, 25, 24, 27,
                                30, 29, 28, 31];

    let p_shuffle: *const __m256i = SHUFFLE.as_ptr() as *const __m256i;
    let p_bgra: *const __m256i = bgra.as_ptr().offset(offset) as *const  __m256i;
    let prgba: *mut __m256i = rgba.as_mut_ptr().offset(offset) as *mut __m256i;
    *prgba = _mm256_shuffle_epi8(*p_bgra, *p_shuffle);
}

#[target_feature(enable="avx2")]
pub unsafe fn avx2_cmp( offset: isize, a: &Vec<u8>, b: &Vec<u8>) -> bool {
    let p_a: *const __m256i = a.as_ptr().offset(offset) as *const __m256i; 
    let p_b: *const __m256i = b.as_ptr().offset(offset) as *const __m256i; 
    
    let mask = _mm256_movemask_epi8( _mm256_cmpeq_epi64(*p_a, *p_b) );

    return match mask as u32 {
        0xffffffff => true,
        _ => false,
    };
}


// if new != old, convert replace old with new, converting from bgra to rgba and return false.
// if new == old, return true (and leave old in an indeterminate state)
#[target_feature(enable="avx2")]
pub unsafe fn avx2_cmp_and_convert( offset: isize, new: &Vec<u8>, old: &mut Vec<u8> ) -> bool {
    const SHUFFLE: [i8; 32] = [2, 1, 0, 3,
                                6, 5, 4,7,
                                10, 9, 8, 11,
                                14, 13, 12, 15,
                                18, 17, 16, 19,
                                22, 21, 20, 23,
                                26, 25, 24, 27,
                                30, 29, 28, 31];
    let p_shuffle: *const __m256i = SHUFFLE.as_ptr() as *const __m256i;
    let p_new: *const __m256i = new.as_ptr().offset(offset) as *const  __m256i;
    let p_old: *mut __m256i = old.as_mut_ptr().offset(offset) as *mut __m256i;
    // compare
    let mask = _mm256_movemask_epi8( _mm256_cmpeq_epi64(*p_new, *p_old) );
    //convert
    *p_old = _mm256_shuffle_epi8(*p_new, *p_shuffle);

    return match mask as u32 {
        0xffffffff => true,
        _ => false,
    };
}


#[target_feature(enable="avx")]
pub unsafe fn ssse3_bgra_to_rgba(offset: isize, bgra: &Vec<u8>, rgba: &mut Vec<u8>) {
    const SHUFFLE: [i8; 16] = [2, 1, 0, 3,
                                6, 5, 4,7,
                                10, 9, 8, 11,
                                14, 13, 12, 15];

    let p_shuffle: *const __m128i = SHUFFLE.as_ptr() as *const __m128i;
    let p_bgra: *const __m128i = bgra.as_ptr().offset(offset) as *const  __m128i ;
    let prgba: *mut __m128i = rgba.as_mut_ptr().offset(offset) as *mut __m128i;
    *prgba = _mm_shuffle_epi8(*p_bgra, *p_shuffle);
}

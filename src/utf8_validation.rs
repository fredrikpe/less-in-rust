use std::mem;

/// Returns the offset of the first valid start of a
/// UTF-8 sequence, if it exists.
/// If the start is already valid, this mean 0.
///
/// Note that the whole sequence doesn't need
/// to be valid, only some beginning sequence.
pub fn first_valid_pos(b: &[u8]) -> Option<usize> {
    let mut offset: usize = 0;

    loop {
        match run_utf8_validation(&b[offset..]) {
            Err(Utf8Error {
                valid_up_to: v,
                error_len: _,
            }) => {
                if v > 0 {
                    return Some(offset);
                }
            }
            Ok(_) => return Some(offset),
        }

        offset += 1;
        if offset >= b.len() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_valid_pos() {
        let s = "พระปกเกศกองบู๊กู้ขึ้นใหม่สิบสองกษัตริย์ก่อนหน้";
        let b = s.as_bytes();
        assert_eq!(first_valid_pos(b), Some(0));
        assert_eq!(first_valid_pos(&b[1..]), Some(2));
        assert_eq!(first_valid_pos(&b[1..129]), Some(2));
        assert_eq!(first_valid_pos(&b[3..129]), Some(0));
        assert_eq!(first_valid_pos(&b[b.len() - 1..]), None);
        assert_eq!(first_valid_pos(&b[0..0]), Some(0));
        assert_eq!(first_valid_pos(&b[0..1]), None);
    }

    #[test]
    fn test_utf8_validation() {
        let s = "พระปกเกศกองบู๊กู้ขึ้นใหม่สิบสองกษัตริย์ก่อนหน้";
        let b = s.as_bytes();
        assert_eq!(run_utf8_validation(b), Ok(()));
        assert!(run_utf8_validation(&b[1..]).is_err());
        assert_eq!(run_utf8_validation(&b[3..129]), Ok(()));
    }
}

/// The following code is copied from the Rust standard library.
/// It is used there internally in for example String::from_utf8
/// and similar methods, but is not public.
///
/// ###########################################################
/// ###########################################################

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct Utf8Error {
    valid_up_to: usize,
    error_len: Option<u8>,
}

// use truncation to fit u64 into usize
const NONASCII_MASK: usize = 0x80808080_80808080u64 as usize;

/// Returns `true` if any byte in the word `x` is nonascii (>= 128).
#[inline]
fn contains_nonascii(x: usize) -> bool {
    (x & NONASCII_MASK) != 0
}

/// Walks through `iter` checking that it's a valid UTF-8 sequence,
/// returning `true` in that case, or, if it is invalid, `false` with
/// `iter` reset such that it is pointing at the first byte in the
/// invalid sequence.
#[inline]
fn run_utf8_validation(v: &[u8]) -> Result<(), Utf8Error> {
    let mut index = 0;
    let len = v.len();

    let usize_bytes = mem::size_of::<usize>();
    let ascii_block_size = 2 * usize_bytes;
    let blocks_end = if len >= ascii_block_size {
        len - ascii_block_size + 1
    } else {
        0
    };

    while index < len {
        let old_offset = index;
        macro_rules! err {
            ($error_len: expr) => {
                return Err(Utf8Error {
                    valid_up_to: old_offset,
                    error_len: $error_len,
                });
            };
        }

        macro_rules! next {
            () => {{
                index += 1;
                // we needed data, but there was none: error!
                if index >= len {
                    err!(None)
                }
                v[index]
            }};
        }

        let first = v[index];
        if first >= 128 {
            let w = UTF8_CHAR_WIDTH[first as usize];
            // 2-byte encoding is for codepoints  \u{0080} to  \u{07ff}
            //        first  C2 80        last DF BF
            // 3-byte encoding is for codepoints  \u{0800} to  \u{ffff}
            //        first  E0 A0 80     last EF BF BF
            //   excluding surrogates codepoints  \u{d800} to  \u{dfff}
            //               ED A0 80 to       ED BF BF
            // 4-byte encoding is for codepoints \u{1000}0 to \u{10ff}ff
            //        first  F0 90 80 80  last F4 8F BF BF
            //
            // Use the UTF-8 syntax from the RFC
            //
            // https://tools.ietf.org/html/rfc3629
            // UTF8-1      = %x00-7F
            // UTF8-2      = %xC2-DF UTF8-tail
            // UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) /
            //               %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )
            // UTF8-4      = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) /
            //               %xF4 %x80-8F 2( UTF8-tail )
            match w {
                2 => {
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        err!(Some(1))
                    }
                }
                3 => {
                    match (first, next!()) {
                        (0xE0, 0xA0..=0xBF)
                        | (0xE1..=0xEC, 0x80..=0xBF)
                        | (0xED, 0x80..=0x9F)
                        | (0xEE..=0xEF, 0x80..=0xBF) => {}
                        _ => err!(Some(1)),
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        err!(Some(2))
                    }
                }
                4 => {
                    match (first, next!()) {
                        (0xF0, 0x90..=0xBF)
                        | (0xF1..=0xF3, 0x80..=0xBF)
                        | (0xF4, 0x80..=0x8F) => {}
                        _ => err!(Some(1)),
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        err!(Some(2))
                    }
                    if next!() & !CONT_MASK != TAG_CONT_U8 {
                        err!(Some(3))
                    }
                }
                _ => err!(Some(1)),
            }
            index += 1;
        } else {
            // Ascii case, try to skip forward quickly.
            // When the pointer is aligned, read 2 words of data per iteration
            // until we find a word containing a non-ascii byte.
            let ptr = v.as_ptr();
            let align = unsafe {
                // the offset is safe, because `index` is guaranteed inbounds
                ptr.add(index).align_offset(usize_bytes)
            };
            if align == 0 {
                while index < blocks_end {
                    unsafe {
                        let block = ptr.add(index) as *const usize;
                        // break if there is a nonascii byte
                        let zu = contains_nonascii(*block);
                        let zv = contains_nonascii(*block.offset(1));
                        if zu | zv {
                            break;
                        }
                    }
                    index += ascii_block_size;
                }
                // step from the point where the wordwise loop stopped
                while index < len && v[index] < 128 {
                    index += 1;
                }
            } else {
                index += 1;
            }
        }
    }

    Ok(())
}

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];

///// Given a first byte, determines how many bytes are in this UTF-8 character.
//#[inline]
//pub fn utf8_char_width(b: u8) -> usize {
//UTF8_CHAR_WIDTH[b as usize] as usize
//}

/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;
/// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte.
const TAG_CONT_U8: u8 = 0b1000_0000;

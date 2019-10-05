use std::i32;

use termion::terminal_size;
use unicode_segmentation::UnicodeSegmentation;

pub fn screen_height_half() -> usize {
    (screen_height() as usize - 1) / 2
}

pub fn screen_width_height() -> (u16, u16) {
    terminal_size().unwrap()
}

pub fn screen_height() -> usize {
    let (_, screen_height) = terminal_size().unwrap();
    screen_height as usize - 1
}

pub fn grapheme_count(buf: &str) -> usize {
    UnicodeSegmentation::graphemes(buf, true).count()
}

pub fn nth_newline_pos(
    mut n: usize,
    buf: &str,
    screen_width: Option<i32>,
) -> usize {
    let mut grapheme_count = 0;
    let mut current_pos = 0;
    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(buf, true) {
        grapheme_count += 1;
        current_pos = offset + grapheme_size(grapheme);
        if is_newline(grapheme)
            || grapheme_count >= screen_width.unwrap_or(i32::MAX)
        {
            grapheme_count = 0;
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }

    current_pos
}

pub fn nth_last_newline_pos(
    n: usize,
    buf: &str,
    screen_width: Option<i32>,
) -> usize {
    let mut offsets = Vec::new();
    let mut grapheme_count = 0;

    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(buf, true) {
        grapheme_count += 1;
        if is_newline(grapheme)
            || grapheme_count >= screen_width.unwrap_or(i32::MAX)
        {
            grapheme_count = 0;
            offsets.push(Some(offset + grapheme_size(grapheme)));
        }
    }

    return if offsets.len() < n {
        0
    } else {
        offsets[offsets.len() - n].unwrap()
    };
}

pub fn is_newline(grapheme: &str) -> bool {
    grapheme == "\n" || grapheme == "\r\n"
}

pub fn grapheme_size(grapheme: &str) -> usize {
    grapheme.len()
}

pub fn make_valid(buf: Vec<u8>) -> Vec<u8> {
    let mut offset = 0;
    while let Err(_) = std::str::from_utf8(&buf[offset..]) {
        offset += 1;
    }

    buf[offset..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snd_last_newline_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        assert_eq!(nth_last_newline_pos(2, s, Some(3)), 0);
        assert_eq!(nth_last_newline_pos(2, t, Some(3)), 0);
        assert_eq!(nth_last_newline_pos(2, u, Some(3)), 1);
        assert_eq!(nth_last_newline_pos(2, v, Some(3)), 3);
    }

    #[test]
    fn test_nth_last_newline_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        let w = "\n\n\n\n\n\n\n\n\n\n";
        let x = "ฤๅหาใครค้ำชูกู้บรรลังก์ ฯ";
        assert_eq!(nth_last_newline_pos(2, s, Some(3)), 0);
        assert_eq!(nth_last_newline_pos(2, t, Some(3)), 0);
        assert_eq!(nth_last_newline_pos(2, u, Some(3)), 1);
        assert_eq!(nth_last_newline_pos(2, v, Some(3)), 3);
        assert_eq!(nth_last_newline_pos(10, w, Some(3)), 1);
        // When we give an incomplete grapheme
        assert!(std::panic::catch_unwind(|| {
            nth_last_newline_pos(56, &x[1..], Some(131));
        })
        .is_err());
        assert!(std::panic::catch_unwind(|| {
            nth_last_newline_pos(56, &x[..x.len() - 1], Some(131));
        })
        .is_err());
    }

    fn bible_string() -> String {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::open("tests/resources/bible_short.txt").unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read the file");
        contents
    }

    #[test]
    fn test_nth_last_newline_wrapped_file() {
        let bible = bible_string();

        for c in bible.chars() {
            eprint!("{} ", c);
        }

        assert_eq!(nth_last_newline_pos(1, &bible[..], Some(3)), 207);
    }

    #[test]
    fn test_nth_newline_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        let x = "\naaaa\naa\n";
        assert_eq!(nth_newline_pos(1, s, Some(3)), 1);
        assert_eq!(nth_newline_pos(1, t, Some(3)), 0);
        assert_eq!(nth_newline_pos(1, u, Some(3)), 1);
        assert_eq!(nth_newline_pos(1, v, Some(3)), 3);
        assert_eq!(nth_newline_pos(2, x, Some(3)), 4);
        assert_eq!(nth_newline_pos(2, s, Some(3)), 1);
    }

    #[test]
    fn test_nth_newline_not_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        let x = "\naaaa\naa\n";
        assert_eq!(nth_newline_pos(1, s, None), 1);
        assert_eq!(nth_newline_pos(1, t, None), 0);
        assert_eq!(nth_newline_pos(1, u, None), 1);
        assert_eq!(nth_newline_pos(1, v, None), 6);
        assert_eq!(nth_newline_pos(2, x, None), 6);
        assert_eq!(nth_newline_pos(2, s, None), 1);
    }
}

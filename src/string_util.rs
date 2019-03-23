use unicode_segmentation::UnicodeSegmentation;

pub fn grapheme_count(buf: &str) -> usize {
    UnicodeSegmentation::graphemes(buf, true).count()
}

#[allow(dead_code)]
pub fn nth_grapheme_offset(buf: &str, n: usize) -> Option<usize> {
    for (index, (offset, _)) in
        UnicodeSegmentation::grapheme_indices(buf, true).enumerate()
    {
        if index == n {
            return Some(offset);
        }
    }

    None
}

pub fn nth_newline_wrapped(
    mut n: usize,
    buf: &str,
    screen_width: usize,
) -> usize {
    let mut grapheme_count = 0;
    let mut current_pos = 0;
    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(buf, true) {
        grapheme_count += 1;
        current_pos = offset + grapheme_size(grapheme);
        if is_newline(grapheme) || grapheme_count >= screen_width {
            grapheme_count = 0;
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }

    current_pos
}

#[allow(dead_code)]
pub fn last_newline_offset(buf: &str) -> Option<usize> {
    let mut last = None;
    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(buf, true) {
        if is_newline(grapheme) {
            last = Some(offset + grapheme_size(grapheme));
        }
    }

    last
}

pub fn nth_last_newline_wrapped(
    n: usize,
    buf: &str,
    screen_width: usize,
) -> usize {
    let mut offsets = Vec::new();
    let mut grapheme_count = 0;

    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(buf, true) {
        grapheme_count += 1;
        if is_newline(grapheme) || grapheme_count >= screen_width {
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

#[allow(dead_code)]
pub fn snd_last_newline_wrapped(buf: &str, screen_width: usize) -> usize {
    nth_last_newline_wrapped(2, buf, screen_width)
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
    fn byte_offset_simple() {
        let s = "Hello!";
        assert_eq!(nth_grapheme_offset(s, 0), Some(0));
        assert_eq!(nth_grapheme_offset(s, 1), Some(1));
        assert_eq!(nth_grapheme_offset(s, 2), Some(2));
        assert_eq!(nth_grapheme_offset(s, 3), Some(3));
        assert_eq!(nth_grapheme_offset(s, 4), Some(4));
        assert_eq!(nth_grapheme_offset(s, 5), Some(5));
        assert_eq!(nth_grapheme_offset(s, 6), None);
    }

    #[test]
    fn byte_offset_strange() {
        let n = "éa";
        let m = "éa";
        assert_eq!(n.len(), m.len() + 1);
        assert_eq!(nth_grapheme_offset(n, 1), Some(n.len() - 1));
        assert_eq!(nth_grapheme_offset(m, 1), Some(m.len() - 1));
    }

    #[test]
    fn test_last_newline_offset() {
        let s = "a\na";
        let t = "\naa";
        let u = "\naa\n";
        let v = "aa";
        assert_eq!(last_newline_offset(s), Some(2));
        assert_eq!(last_newline_offset(t), Some(1));
        assert_eq!(last_newline_offset(u), Some(4));
        assert_eq!(last_newline_offset(v), None);
    }

    #[test]
    fn test_snd_last_newline_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        assert_eq!(snd_last_newline_wrapped(s, 3), 0);
        assert_eq!(snd_last_newline_wrapped(t, 3), 0);
        assert_eq!(snd_last_newline_wrapped(u, 3), 1);
        assert_eq!(snd_last_newline_wrapped(v, 3), 3);
    }

    #[test]
    fn test_nth_last_newline_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        let w = "\n\n\n\n\n\n\n\n\n\n";
        let x =
            "ฤๅหาใครค้ำชูกู้บรรลังก์ ฯ";
        assert_eq!(nth_last_newline_wrapped(2, s, 3), 0);
        assert_eq!(nth_last_newline_wrapped(2, t, 3), 0);
        assert_eq!(nth_last_newline_wrapped(2, u, 3), 1);
        assert_eq!(nth_last_newline_wrapped(2, v, 3), 3);
        assert_eq!(nth_last_newline_wrapped(10, w, 3), 1);
        // When we give an incomplete grapheme
        assert!(std::panic::catch_unwind(|| {
            nth_last_newline_wrapped(56, &x[1..], 131);
        })
        .is_err());
        assert!(std::panic::catch_unwind(|| {
            nth_last_newline_wrapped(56, &x[..x.len() - 1], 131);
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

        assert_eq!(nth_last_newline_wrapped(1, &bible[..], 3), 207);
    }

    #[test]
    fn test_nth_newline_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        let x = "\naaaa\naa\n";
        assert_eq!(nth_newline_wrapped(1, s, 3), 1);
        assert_eq!(nth_newline_wrapped(1, t, 3), 0);
        assert_eq!(nth_newline_wrapped(1, u, 3), 1);
        assert_eq!(nth_newline_wrapped(1, v, 3), 3);
        assert_eq!(nth_newline_wrapped(2, x, 3), 4);
        assert_eq!(nth_newline_wrapped(2, s, 3), 1);
    }
}

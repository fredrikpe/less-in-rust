use unicode_segmentation::GraphemeIndices;
use unicode_segmentation::UnicodeSegmentation;

pub fn nth_grapheme_offset(buf: &str, n: usize) -> Option<usize> {
    for (index, (offset, _)) in UnicodeSegmentation::grapheme_indices(buf, true).enumerate() {
        if index == n {
            return Some(offset);
        }
    }

    None
}

pub fn next_newline_offset(buf: &str) -> Option<usize> {
    find_offset(
        UnicodeSegmentation::grapheme_indices(buf, true),
        |x: &str| is_newline(x),
    )
}

pub fn aprev_newline_offset(buf: &str) -> Option<usize> {
    find_offset(
        UnicodeSegmentation::grapheme_indices(buf, true).rev(),
        |x: &str| is_newline(x),
    )
}

pub fn find_offset<'a, P, I: Iterator<Item = (usize, &'a str)>>(
    grapheme_indices: I,
    mut predicate: P,
) -> Option<usize>
where
    P: FnMut(&str) -> bool,
{
    for (offset, grapheme) in grapheme_indices {
        if predicate(grapheme) {
            return Some(offset);
        }
    }

    None
}

pub fn nth_newline_wrapped(mut n: usize, buf: &str, screen_width: usize) -> Option<usize> {
    let mut grapheme_count = 0;
    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(buf, true) {
        grapheme_count += 1;
        if is_newline(grapheme) || grapheme_count >= screen_width {
            grapheme_count = 0;
            n -= 1;
            if n == 0 {
                return Some(offset + grapheme_size(grapheme));
            }
        }
    }

    None
}

pub fn first_newline_wrapped(buf: &str, screen_width: usize) -> Option<usize> {
    nth_newline_wrapped(1, buf, screen_width)
}

pub fn last_newline_wrapped(buf: &str, screen_width: usize) -> Option<usize> {
    let mut last = Some(0);
    for (index, (offset, grapheme)) in
        UnicodeSegmentation::grapheme_indices(buf, true).enumerate()
    {
        if is_newline(grapheme) || index >= screen_width {
            last = Some(offset);
        }
    }

    last
}

pub fn last_newline_offset(buf: &str) -> Option<usize> {
    let mut last = None;
    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(buf, true) {
        if is_newline(grapheme) {
            last = Some(offset + grapheme_size(grapheme));
        }
    }

    last
}

pub fn nth_last_newline_wrapped(n: usize, buf: &str, screen_width: usize) -> usize {
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
        eprintln!("nth 0");
        0
    } else {
        eprintln!("nth {}", offsets[offsets.len() - n].unwrap());
        offsets[offsets.len() - n].unwrap()
    };
}

pub fn snd_last_newline_wrapped(buf: &str, screen_width: usize) -> usize {
    nth_last_newline_wrapped(2, buf, screen_width)
}

pub fn is_newline(grapheme: &str) -> bool {
    grapheme == "\n"
}

pub fn grapheme_size(grapheme: &str) -> usize {
    grapheme.len()
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
        assert_eq!(nth_last_newline_wrapped(2, s, 3), 0);
        assert_eq!(nth_last_newline_wrapped(2, t, 3), 0);
        assert_eq!(nth_last_newline_wrapped(2, u, 3), 1);
        assert_eq!(nth_last_newline_wrapped(2, v, 3), 3);
        assert_eq!(nth_last_newline_wrapped(10, w, 3), 1);
    }

    #[test]
    fn test_first_newline_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        assert_eq!(first_newline_wrapped(s, 3), Some(1));
        assert_eq!(first_newline_wrapped(t, 3), None);
        assert_eq!(first_newline_wrapped(u, 3), Some(1));
        assert_eq!(first_newline_wrapped(v, 3), Some(3));
    }

    #[test]
    fn test_nth_newline_wrapped() {
        let s = "\n";
        let t = "";
        let u = "\naa\n";
        let v = "aaaaaa";
        let w = "\n\n\n\n\n\n\n\n";
        let x = "\naaaa\naa\n";
        assert_eq!(nth_newline_wrapped(1, s, 3), Some(1));
        assert_eq!(nth_newline_wrapped(1, t, 3), None);
        assert_eq!(nth_newline_wrapped(1, u, 3), Some(1));
        assert_eq!(nth_newline_wrapped(1, v, 3), Some(3));
        assert_eq!(nth_newline_wrapped(2, x, 3), Some(4));
    }
}

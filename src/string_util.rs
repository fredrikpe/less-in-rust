use unicode_segmentation::UnicodeSegmentation;

pub fn nth_grapheme_offset(buf: &str, n: usize) -> Option<usize> {
    for (index, (offset, _)) in UnicodeSegmentation::grapheme_indices(buf, true).enumerate() {
        if index == n {
            return Some(offset);
        }
    }

    None
}

pub fn first_newline_offset(buf: &str, screen_width: usize) -> Option<usize> {
    for (index, (offset, grapheme))
        in UnicodeSegmentation::grapheme_indices(buf, true).enumerate()
    {
        if is_newline(grapheme) || index >= screen_width {
            return Some(offset);
        }
    }

    None
}

/*
pub fn first_newline_offset(buf: &str, screen_width: usize) -> Option<usize> {
    let mut grapheme_count = 0;
    for (offset, grapheme) in UnicodeSegmentation::grapheme_indices(buf, true) {
        if is_newline(grapheme) || grapheme_count >= screen_width as usize {
            return Some(offset);
        }

        if is_newline(grapheme) {
            grapheme_count = 0;
            screen_line_number += 1;
            writeln!(self.out);
            write!(self.out, "{}", termion::cursor::Goto(1, screen_line_number));
        } else {
            grapheme_count += 1;
            write!(self.out, "{}", grapheme);
        }
    }
}
*/

pub fn is_newline(grapheme: &str) -> bool {
    grapheme == "\n"
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
}

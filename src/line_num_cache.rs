

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LineNum {
    pub num: i32,
    pub offset: usize,
}

pub struct LineNumCache {
    pub line_nums: [Option<LineNum>; 24],
}

impl LineNumCache {
    pub fn new() -> LineNumCache {
        LineNumCache {
            line_nums: [None; 24],
        }
    }

    pub fn add(&mut self, line_num: LineNum) {
        match self.try_add(line_num) {
            Ok(_) => return (),
            Err(_) => self.add_important(line_num),
        }
    }

    pub fn prev(&mut self, offset: usize) -> Option<(LineNum, i64)> {
        let mut prev = None;
        let mut distance = std::i64::MAX;
        for i in 0..24 {
            if let Some(ln) = self.line_nums[i] {
                let d = offset as i64 - ln.offset as i64;
                if d > 0 && d < distance {
                    distance = d;
                    prev = Some((ln, d));
                }
            }
        }
        prev
    }

    pub fn closest(&mut self, offset: usize) -> Option<(LineNum, i64)> {
        let mut closest = None;
        let mut distance = std::i64::MAX;
        for i in 0..24 {
            if let Some(ln) = self.line_nums[i] {
                let d =  ln.offset as i64 - offset as i64;
                if d.abs() < distance {
                    distance = d.abs();
                    closest = Some((ln, d));
                }
            }
        }
        closest
    }

    // TODO 
    fn add_important(&mut self, line_num: LineNum) {
        self.line_nums[0] = Some(line_num)
    }

    fn try_add(&mut self, line_num: LineNum) -> Result<(), ()> {
        for i in 0..24 {
            if self.line_nums[i].is_none() {
                self.line_nums[i] = Some(line_num);
                return Ok(());
            }
        }
        Err(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s() {
        let mut s = LineNumCache::new();
        s.add(LineNum {num: 24, offset: 34});

        assert_eq!(s.line_nums[0].is_some(), true);
        assert_eq!(s.line_nums[1].is_none(), true);
    }
 
    #[test]
    fn closest_none() {
        let mut s = LineNumCache::new();
        assert!(s.closest(0).is_none());
    }   

    #[test]
    fn closest() {
        let mut s = LineNumCache::new();
        let l = LineNum {num: 24, offset: 34};
        let m = LineNum {num: 8, offset: 2};
        let n = LineNum {num: 389, offset: 1000};
        s.add(l);
        s.add(m);
        s.add(n);

        assert_eq!(s.closest(999), Some((n, 1)));
        assert_eq!(s.closest(10), Some((m, -8)));
        assert_eq!(s.closest(30), Some((l, 4)));
    }   
}



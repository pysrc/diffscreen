use std::fmt::Display;

pub struct Bitmap(u128, u128);

impl Bitmap {
    pub fn new() -> Self {
        Bitmap(0, 0)
    }
    pub fn push(&mut self, key: u8) -> bool {
        if key <= 127 {
            // 0-127
            let b = 1 << key;
            if self.1 & b == b {
                return false;
            }
            self.1 |= b;
        } else {
            // 128-255
            let b = 1 << (key - 128);
            if self.0 & b == b {
                return false;
            }
            self.0 |= b;
        }
        return true;
    }

    pub fn remove(&mut self, key: u8) {
        if key <= 127 {
            let b = !(1 << key);
            self.1 &= b;
        } else {
            let b = !(1 << (key - 128));
            self.0 &= b;
        }
    }
}

impl Display for Bitmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "({:b}, {:b})", self.0, self.1)
    }
}

#[test]
fn test() {
    let mut bm = Bitmap::new();
    assert_eq!(bm.push(0), true);
    assert_eq!(bm.push(10), true);
    assert_eq!(bm.push(127), true);
    assert_eq!(bm.push(128), true);
    assert_eq!(bm.push(168), true);
    assert_eq!(bm.push(255), true);

    assert_eq!(bm.push(0), false);
    assert_eq!(bm.push(10), false);
    assert_eq!(bm.push(127), false);
    assert_eq!(bm.push(128), false);
    assert_eq!(bm.push(168), false);
    assert_eq!(bm.push(255), false);

    bm.remove(10);
    bm.remove(168);

    assert_eq!(bm.push(10), true);
    assert_eq!(bm.push(168), true);
}

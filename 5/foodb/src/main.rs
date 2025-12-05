struct ClosedInt(u32, u32);

impl ClosedInt {
    fn contains(&self, num: u32) -> bool {
        num >= self.0 && num <= self.1
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let closed = ClosedInt(10, 15);
        for i in 10..15 {
            assert!(closed.contains(i));
        }
        assert!(!closed.contains(9));
        assert!(!closed.contains(16));
    }
}

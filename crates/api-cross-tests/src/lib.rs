pub fn xyz() -> usize { 42 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xyz() {
        assert_eq!(xyz(), 42);
    }
}

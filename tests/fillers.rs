#![allow(dead_code)]

#[path = "../src/utils.rs"]
mod utils;

#[cfg(test)]
mod filler {
    use crate::utils;

    #[test]
    fn filler_with_zero_max_and_used() {
        assert_eq!(utils::filler("-", 0, 0), "");
    }

    #[test]
    fn filler_with_zero_used() {
        assert_eq!(utils::filler("-", 5, 0), "-----");
    }

    #[test]
    fn filler_with_zero_max() {
        assert_eq!(utils::filler("-", 0, 5), "");
    }

    #[test]
    fn filler_with_same_max_and_used() {
        assert_eq!(utils::filler("-", 5, 5), "");
    }

    #[test]
    fn filler_with_more_used_than_max() {
        assert_eq!(utils::filler("-", 5, 10), "");
    }
}

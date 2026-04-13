use proptest::prelude::*;

proptest! {
    #[test]
    fn test_shitty(s in ".*") {
        assert!(s.len() >= 0);
    }
}

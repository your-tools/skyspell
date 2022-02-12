// Note: these tests will fail if for some reason 'missstake' is in
// the personal dict, or if no Enchant provider for the US English dictionary is found,
// and there's no good way to know :(
//
#[macro_export]
macro_rules! test_dictionary {
    ($dictionary:ty) => {
        use crate::Dictionary;

        #[test]
        fn test_check_valid_word() {
            let dict = <$dictionary>::new("en_US").unwrap();
            assert!(!dict.check("missstake").unwrap());
        }
    };
}

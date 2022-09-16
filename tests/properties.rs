use proptest::{prop_assert, proptest, string};
use sais::{sais, sais_utf8};

proptest! {
    #[test]
    fn valid_bytes(s in string::bytes_regex(".*").unwrap()) {
        let sa = sais(&s);
        for neighbors in sa.windows(2) {
            prop_assert!(&s[neighbors[0]..] < &s[neighbors[1]..]);
        }
    }

    #[test]
    fn valid_utf8(s in ".*") {
        let sa = sais_utf8(&s);
        for neighbors in sa.windows(2) {
            prop_assert!(&s[neighbors[0]..] < &s[neighbors[1]..]);
        }
    }
}

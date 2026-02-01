//! Tests for lifetime parameters in relate_structs!

use relate::relate_structs;

// Test basic lifetime parameter - converting borrowed to owned
mod basic_lifetime {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Borrowed<'a> {
        text: &'a str,
        num:  i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct OwnedCopy {
        text: String,
        num:  i32,
    }

    relate_structs! {
        Borrowed<'a> ~> OwnedCopy {
            text: with = _.to_string();
            num;
        }
    }

    #[test]
    fn test_borrowed_to_owned() {
        let text = "hello world";
        let borrowed = Borrowed { text, num: 42 };
        let owned: OwnedCopy = borrowed.into();
        assert_eq!(owned.text, "hello world");
        assert_eq!(owned.num, 42);
    }

    #[test]
    fn test_from_ref() {
        let text = "reference";
        let borrowed = Borrowed { text, num: 10 };
        let owned: OwnedCopy = (&borrowed).into();
        assert_eq!(owned.text, "reference");
    }
}

// Test multiple lifetimes
mod multi_lifetime {
    use super::*;

    #[derive(Debug, Clone)]
    struct TwoBorrows<'a, 'b> {
        first:  &'a str,
        second: &'b str,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TwoOwned {
        first:  String,
        second: String,
    }

    relate_structs! {
        TwoBorrows<'a, 'b> ~> TwoOwned {
            first: with = _.to_string();
            second: with = _.to_string();
        }
    }

    #[test]
    fn test_multi_lifetime() {
        let s1 = "first";
        let s2 = "second";
        let borrowed = TwoBorrows {
            first:  s1,
            second: s2,
        };
        let owned: TwoOwned = borrowed.into();
        assert_eq!(owned.first, "first");
        assert_eq!(owned.second, "second");
    }
}

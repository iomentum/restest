fn main() {
    restest::assert_body_matches! {
            42
        ,
        val
    };

    // val is in scope
    assert_eq!(val, 42isize);

    restest::assert_body_matches! {
            [42]
        ,
        [val_]
    };

    // val_ is in scope
    assert_eq!(val_, 42isize);

    restest::assert_body_matches! {
            42
        ,
        val__
    };

    assert_eq!(val__, 42u8);
}

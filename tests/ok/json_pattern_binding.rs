fn main() {
    restest::assert_body_matches! {
        serde_json::json! {
            42
        },
        val as isize
    };

    // val is in scope
    assert_eq!(val, 42isize);

    restest::assert_body_matches! {
        serde_json::json! {
            [42]
        },
        [val_ as isize]
    };

    // val_ is in scope
    assert_eq!(val_, 42isize);

    restest::assert_body_matches! {
        serde_json::json! {
            42
        },
        val__ as u8
    };

    assert_eq!(val__, 42u8);
}

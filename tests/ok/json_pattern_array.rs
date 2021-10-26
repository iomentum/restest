fn main() {
    restest::assert_body_matches!([42, 41], [42, 41]);

    restest::assert_body_matches! {
        [42, 101],
        a
    };

    assert_eq!(a, [42, 101]);

    restest::assert_body_matches! {
        [101, 42],
        [a, 42],
    };

    assert_eq!(a, 101);
}

fn main() {
    restest::assert_body_matches!(vec![42, 41], [42, 41]);

    restest::assert_body_matches! {
        vec![42, 101],
        a
    };

    assert_eq!(a, [42, 101]);

    restest::assert_body_matches! {
        vec![101, 42],
        [a, 42],
    };

    assert_eq!(a, 101);

    restest::assert_body_matches! {
        vec![101, 102],
        [a, _],
    };

    assert_eq!(a, 101);
}

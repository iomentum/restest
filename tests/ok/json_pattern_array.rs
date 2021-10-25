fn main() {
    restest::assert_body_matches!(
        serde_json::json! {
            [42, 41]
        },
        [42, 41]
    );

    restest::assert_body_matches! {
        serde_json::json! {
            [42, 101]
        },
        a as [isize]
    };

    assert_eq!(a, [42, 101]);
}

fn main() {
    restest::assert_body_matches! {
        serde_json::json! {
            "foo"
        },
        "foo"
    };

    restest::assert_body_matches! {
        serde_json::json! {
            "bar"
        },
        "bar"
    };
}

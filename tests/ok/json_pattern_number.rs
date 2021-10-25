fn main() {
    restest::assert_body_matches!(serde_json::json! { 42 }, 42);
}

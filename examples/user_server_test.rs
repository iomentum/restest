use restest::assert_api;
use serde::Serialize;

#[derive(Serialize)]
struct UsersInput {
    year_of_birth: usize,
}

#[test]
fn put_simple() {
    assert_api! {
        PUT "/users",
        UsersInput {
            year_of_birth: 2000,
        } => User {
            year_of_birth,
        }
    }
}

fn main() {
    panic!("Usage: cargo test --example user_server_test");
}
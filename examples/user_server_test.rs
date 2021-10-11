use restest::assert_api;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
struct UserInput {
    year_of_birth: usize,
}

#[derive(Deserialize)]
struct User {
    year_of_birth: usize,
    id: Uuid,
}

#[test]
pub fn put_simple() {
    assert_api! {
        PUT "/users",
        UsersInput {
            year_of_birth: 2000,
        } => User {
            year_of_birth: 2000,
            ..
        }
    };
}

fn main() {
    panic!("Usage: cargo test --example user_server_test");
}
use http::StatusCode;
use restest::request::Request;
use serde::{Deserialize, Serialize};

const CONTEXT: restest::Context = restest::Context::new()
    .with_port(4242)
    .with_host("http://localhost");

#[tokio::test]
async fn test_user_addition() {
    // First we add a new user to the backend database
    let request = Request::post("users")
        .with_header("token", "mom_said_yes")
        .with_body(AddUserInput {
            name: "Ada Lovelace".to_string(),
            age: 36,
        });

    let user = CONTEXT
        .run(request)
        .await
        .expect_status(StatusCode::OK)
        .deserialize::<User>()
        .await;

    assert_eq!(user.name, "Ada Lovelace");
    assert_eq!(user.age, 36);

    let id = user.id;

    println!("Ada has id `{}`", id);

    // Then we query the backend with the same id to ensure that nothing has
    // changed.
    let request = Request::get(restest::path!["users", id])
        .with_header("token", "mom_said_yes")
        .with_body(());

    let user = CONTEXT
        .run(request)
        .await
        .expect_status(StatusCode::OK)
        .deserialize::<User>()
        .await;

    assert_eq!(user.name, "Ada Lovelace");
    assert_eq!(user.age, 36);
}

#[derive(Serialize)]
struct AddUserInput {
    name: String,
    age: usize,
}

#[derive(Deserialize)]
struct User {
    name: String,
    age: usize,
    id: usize, // Technically very bad, but let's pretend it is ok
}

fn main() {
    panic!("Usage: $ cargo run --example v2_api_example");
}

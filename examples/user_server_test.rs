//! # User server test example
//!
//! *To find the quick start commands, scroll at the bottom of the
//! documentation.*
//!
//! ## Description
//!
//! We're testing a small user database REST API using `restest`-provided
//! macros.
//!
//! ## Testing the API
//!
//! The server must be started in a terminal, so that the testing code can query
//! it. The server never ends, so it must be stopped by pressing Ctrl + C.
//!
//! The nightly toolchain must be used to compile the test code, but the server
//! can be compiled with any toolchain. In order to avoid recompiling all the
//! `dev-dependencies` each time, it is better to compile everything with the
//! nightly toolchain.
//!
//! ## Commands
//!
//! $ cargo run --example user_server
//! $ cargo test --example user_server_test

// I'm sorry but I hate having so much warnings when checking the codebase.
#![allow(dead_code, unused_imports)]

use http::StatusCode;
use restest::{assert_body_matches, path, Context, Request};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The data we send to the server when using the `PUT` route.
///
/// This does not need to be *exactly* the same as the datatype defined in
/// `user_server`. The only constraint is that this must `Serialize` to a JSON
/// body that is accepted by the server.
#[derive(Serialize)]
struct UserInput {
    year_of_birth: usize,
}

/// The data that the server sends us back when we add an user.
///
/// Once again, the only constraint is that we must deserialize what the server
/// responds to us.
#[derive(Debug, Deserialize)]
struct User {
    year_of_birth: usize,
    id: Uuid,
}

/// Let's tell to restest which port should be used for our tests:
const CONTEXT: Context = Context::new().with_port(8080);

/// Test POST route.
///
/// We send a request adding a new user to the database, and tell what we expect
/// as a response.
#[tokio::test]
pub async fn post_user() {
    // Let's create a Request object, representing what we're about to ask to
    // the server.
    let request = Request::post("users").with_body(UserInput {
        year_of_birth: 2000,
    });

    // Now that we have our request object, we can ask our Context to run it.
    //
    // We also check that the response status is what we expect and deserialize
    // the body.
    let user = CONTEXT
        .run(request)
        .await
        .expect_status(StatusCode::CREATED)
        .await;

    assert_body_matches! {
        user,
        User { year_of_birth: 2000, .. },
    };
}

/// Test for the GET route.
///
/// We add a new user to the database and get again its profile so that we
/// can ensure that both profiles are equal.
#[tokio::test]
pub async fn get_user() {
    // Create a new Request object, just as we did for the post_user test.
    let request = Request::post("users").with_body(UserInput {
        year_of_birth: 2000,
    });

    // Similarly, execute the said request and get the output.
    let user = CONTEXT
        .run(request)
        .await
        .expect_status(StatusCode::CREATED)
        .await;

    // Here is a little trick: we need to get back the user ID so that we can
    // reuse it for the next request. To do so, we bind the variable id to the
    // field id of the object we got in response.
    assert_body_matches! {
        user,
        User { id, year_of_birth: 2000 },
    };

    // We can now use the id variable to create another request.
    let request = Request::get(path!["users", id]).with_body(());

    let response = CONTEXT
        .run(request)
        .await
        .expect_status(StatusCode::OK)
        .await;

    // We can ensure that the returned year of birth is now correct.
    assert_body_matches! {
        response,
        User { year_of_birth: 2000, .. },
    };
}

/// Test for the DELETE route.
///
/// We add a new user to the database, then delete it and ensure that the
/// server returns a 200 status code.
///
/// We then try to delete the same user again and ensure that the server
/// returns a 404 status code.
#[tokio::test]
pub async fn delete_user() {
    // Create a new Request object, just as we did for the post_user test.
    let request = Request::post("users").with_body(UserInput {
        year_of_birth: 2000,
    });

    // Similarly, execute the said request and get the output.
    let user = CONTEXT
        .run(request)
        .await
        .expect_status(StatusCode::CREATED)
        .await;

    assert_body_matches! {
        user,
        User { id, year_of_birth: 2000 },
    };

    let request = Request::delete(path!["users", id]).with_body(());

    CONTEXT
        .run(request)
        .await
        .expect_status::<String>(StatusCode::OK)
        .await;

    // We try to delete the same user again and ensure that the server
    // returns a 404 status code.
    let request = Request::delete(path!["users", id]).with_body(());

    CONTEXT
        .run(request)
        .await
        .expect_status::<String>(StatusCode::NOT_FOUND)
        .await;
}

/// Test for the PUT route.
///
/// We add a new user to the database, then update its profile and ensure that
/// the server returns a 200 status code.
#[tokio::test]
pub async fn put_user() {
    // Create a new Request object, just as we did for the post_user test.
    let request = Request::post("users").with_body(UserInput {
        year_of_birth: 2000,
    });

    // Similarly, execute the said request and get the output.
    let user = CONTEXT
        .run(request)
        .await
        .expect_status(StatusCode::CREATED)
        .await;

    assert_body_matches! {
        user,
        User { id, year_of_birth: 2000 },
    };

    // We can now use the id variable to create another request.
    let request = Request::put(path!["users", id]).with_body(UserInput {
        year_of_birth: 2001,
    });

    let response = CONTEXT
        .run(request)
        .await
        .expect_status(StatusCode::OK)
        .await;

    // We can ensure that the returned year of birth is now correct.
    assert_body_matches! {
        response,
        User { year_of_birth: 2001, .. },
    };
}

fn main() {
    panic!("Usage: cargo test --example user_server_test");
}

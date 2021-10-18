//! # User server test example
//!
//! *To find the quick start commands, scroll at the bottom of the
//! documentation.*
//!
//! ## Description
//!
//! We're testing a simple user database REST API using `restest`-provided
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
//! $ cargo +nightly run --example user_server
//! $ cargo +nightly test --example user_server_test

#![feature(assert_matches, let_else)]

use restest::assert_api;
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
    // This field is here to show that we can omit fields when running the
    /// `assert_api` macro, but is never read thought.
    id: Uuid,
}

// Let's tell to restest which port should be used for our tests:
restest::port! { 8080 }

/// A simple test for the PUT route.
///
/// We send a simple request adding a new user to the database, and tell what
/// we expect as a response.
#[tokio::test]
pub async fn put_simple() {
    // The first macro argument is the type of request, followed by the URL.
    //
    // Next to it are the input and expected output. The input is a plain
    // expression that must be `Serialize` and the output is a *pattern* that
    // is compared with the response body.
    //
    // We're using a pattern instead of a regular expression so that some
    // fields can be omitted. Similarly, we can include a specific variant
    // without caring about the others.
    assert_api! {
        POST "/users",
        UserInput {
            year_of_birth: 2000,
        } => User {
            year_of_birth: 2000,
            ..
        }
    };
}

/// A simple test for the GET route.
///
/// We add a new user to the database and get again its profile so that we
/// can ensure that both profiles are equal.
#[tokio::test]
pub async fn get_simple() {
    // Add an user and bind variable id to the user id.
    assert_api! {
        POST "users",
        UserInput {
            year_of_birth: 42,
        } => User {
            id,
            ..
        }
    };

    // Retrieve user whose id is equal to the content of the variable id:
    assert_api! {
        GET ["users", id],
        () => User {
            year_of_birth,
            ..
        }
    };

    assert_eq!(year_of_birth, 42);
}

fn main() {
    panic!("Usage: cargo test --example user_server_test");
}

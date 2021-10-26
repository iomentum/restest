//! # User server example
//!
//! *To find the quick start commands, scroll at the bottom of the
//! documentation.*
//!
//! ## Description
//!
//! Let's model a small server which stores information about its users.
//! More specifically, it stores their id (an Uuid) and year of birth.
//! It exposes a REST API, where we can add a new user and query data about
//! an user with a specific id.
//!
//! To simplify the thing, we save everything in memory.
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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{body, filters::method, path, reply, Filter, Rejection, Reply};

/// An in-memory user database.
#[derive(Clone, Debug, Default)]
struct Database {
    /// We use a very complex type here because we need to share the hashmap
    /// across multiple threads and we want to allow concurrent modifications.
    users: Arc<Mutex<HashMap<Uuid, UserInfos>>>,
}

impl Database {
    fn new() -> Self {
        Self::default()
    }

    fn post_route(self) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        method::post()
            .and(body::json::<UserInfosInput>())
            .map(move |input| {
                let id = Uuid::new_v4();
                let user_infos = Self::make_user(id, input);

                let response = reply::json(&user_infos);
                self.users.lock().unwrap().insert(id, user_infos);
                response
            })
    }

    fn get_route(self) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        method::get()
            .and(path::param())
            .map(move |id| match self.users.lock().unwrap().get(&id) {
                Some(user) => reply::json(user),
                None => reply::json(&"Failed to get user infos"),
            })
    }

    fn make_user(id: Uuid, input: UserInfosInput) -> UserInfos {
        UserInfos {
            id,
            year_of_birth: input.year_of_birth,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct UserInfos {
    id: Uuid,
    /// Let's consider that everyone who's born before 0AD is dead now.
    year_of_birth: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct UserInfosInput {
    year_of_birth: usize,
}

#[tokio::main]
async fn main() {
    let db = Database::new();

    let post = path::path("users").and(db.clone().post_route());
    let get = path::path("users").and(db.get_route());

    let server = warp::serve(post.or(get)).run(([127, 0, 0, 1], 8080));

    server.await
}

//! Let's model a simple server which stores information about its users.
//! More specifically, it stores their id (an Uuid) and year of birth.
//! It exposes a REST API, where we can add a new user and query data about
//! an user with a specific id.
//! 
//! To simplify the thing, we save everything in memory.

use std::{collections::HashMap, sync::{Arc, Mutex}};

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply, body, filters::method, path, reply};

fn post_route(db: Database) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone  {
    method::post().and(body::json::<UserInfosInput>()).map(move |input| {
        let id = Uuid::new_v4();
        let user_infos = make_user(id, input);

        let response = reply::json(&user_infos);
        db.users.lock().unwrap().insert(id, user_infos);
        response
    })
}

fn make_user(id: Uuid, input: UserInfosInput) -> UserInfos {
    UserInfos {
        id,
        year_of_birth: input.year_of_birth,
    }
}

fn get_route(db: Database) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone  {
    method::get().and(path::param()).map(move |id| {
        match db.users.lock().unwrap().get(&id) {
            Some(user) => reply::json(user),
            None => reply::json(&"Failed to get user infos"),
        }
    })
}

#[derive(Clone, Debug, Default)]
struct Database {
    users: Arc<Mutex<HashMap<Uuid, UserInfos>>>,
}

impl Database {
    fn new() -> Self {
        Self::default()
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

    let post = path::path("users").and(post_route(db.clone()));
    let get = path::path("users").and(get_route(db));

    let server = warp::serve(post.or(get)).run(([127, 0, 0, 1], 8080));

    server.await
}
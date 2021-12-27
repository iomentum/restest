# `restest`

Black-box integration test for REST APIs in Rust.

`restest` provides primitives that allow to write REST API in a declarative
manner. It leverages the Rust test framework and uses macro-assisted pattern
tho assert for a pattern and add specified variables to scope.

## Adding to the `Cargo.toml`

`restest` provides test-only code. As such, it can be added as a
dev-dependency:

```TOML
[dev-dependencies]
restest = "0.1.0"
```

## Example

```rust
use restest::{assert_body_matches, path, Context, Request};

use serde::{Deserialize, Serialize};
use http::StatusCode;

const CONTEXT: Context = Context::new().with_port(8080);

let request = Request::post(path!["user/ghopper"]).with_body(PostUser {
    year_of_birth: 1943,
});

let body = CONTEXT
    .run(request)
    .await
    .expect_status(StatusCode::OK)
    .await;

assert_body_matches! {
    body,
    User {
        year_of_birth: 1943,
        ..
    }
}

struct PostUser {
    year_of_birth: usize,
}

struct User {
    year_of_birth: usize,
    id: Uuid
}
```

## Writing tests

Writing tests using `restest` always follow the same patterns. They are
described below.

### Specifying the context

The `Context` object handles all the server-specific configuration:

* which base URL should be used,
* which port should be used.

It can be created with `Context::new`. All its setters are `const`, so
it can be initialized once for all the tests of a module:

```rust
use restest::Context;

const CONTEXT: Context = Context::new().with_port(8080);

async fn test_first_route() {
    // Test code that use `CONTEXT` for a specific route
}

async fn test_second_route() {
    // Test code that use `CONTEXT` again for another route
}
```

As we're running `async` code under the hood, all the tests must be `async`,
hence the use of `tokio::test`

## Creating a request

Let's focus on the test function itself.

The first thing to do is to create a `Request` object. This object allows
to specify characteristics about a specific request that is performed later.

Running `Request::get` allows to construct a GET request to a specific
URL. Header keys can be specified by calling the `with_header` method. The
final `Request` is built by calling the `with_body` method, which allows to add
a body.

```rust
use restest::{path, Request};

let request = Request::get(path!["users", "scrabsha"])
    .with_header("token", "mom-said-yes")
    .with_body(());
```

Similarly, POST requests can be creating by using \[`Request::post`\] instead
of \[`Request::get`\].

## Performing the request

Once that the `Request` object has been created, we can run the request by
passing the `Request` to the `Context` when calling `Context::run`. Once
`await`-ed, the `expect_status` method checks for the request status code and
converts the response body to the expected output type.

```rust
use http::StatusCode;
use uuid::Uuid;
use serde::Deserialize;

let user: User = CONTEXT
    .run(request)
    .await
    .expect_status(StatusCode::OK)
    .await;

struct User {
    name: String,
    age: u8,
    id: Uuid,
}
```

## Checking the response body

Properties about the response body can be asserted with `assert_body_matches`.
The macro supports the full rust pattern syntax, making it easy to check for
expected values and variants. It also provides bindings, allowing you to bring
data from the body in scope:

```rust
use restest::assert_body_matches;

assert_body_matches! {
    user,
    User {
        name: "Grace Hopper",
        age: 85,
        id,
    },
}

// id is now a variable that can be used:
println!("Grace Hopper has id `{}`", id);
```

The extracted variable can be used for next requests or more complex
testing.

*And that's it!*

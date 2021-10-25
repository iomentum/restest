fn main() {
    restest::assert_body_matches! {
        serde_json::json! {
            42
        },
        val as isize
    };

    // val is in scope
    assert_eq!(val, 42isize);

    restest::assert_body_matches! {
        serde_json::json! {
            [42]
        },
        [val_ as isize]
    };

    // val_ is in scope
    assert_eq!(val_, 42isize);

    restest::assert_body_matches! {
        serde_json::json! {
            42
        },
        val__ as u8
    };

    assert_eq!(val__, 42u8);

    let uuid = uuid::Uuid::new_v4();

    restest::assert_body_matches! {
        serde_json::json! {
            {
                "name": "Grace Hopper",
                "id": uuid.to_string(),
            }
        },
        user as User
    }

    assert_eq!(user.name, "Grace Hopper");
    assert_eq!(user.id, uuid);
}

#[derive(serde::Deserialize)]
struct User {
    name: String,
    id: uuid::Uuid,
}

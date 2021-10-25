fn main() {
    restest::assert_body_matches! {
        serde_json::json! {
            {
                "name": "John Doe",
            }
        },
        {
            name: "John Doe",
        }
    };

    restest::assert_body_matches! {
        serde_json::json! {
            {
                "name": "John Doe",
            }
        },
        {
            name: name_ as String,
        }
    };

    assert_eq!(name_, "John Doe");

    restest::assert_body_matches! {
        serde_json::json! {
            {
                "age": 42,
            }
        },
        {
            age: age_ as usize,
        }
    }

    assert_eq!(age_, 42);
}

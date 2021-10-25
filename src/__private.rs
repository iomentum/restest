use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde_json::{Map, Number, Value};

#[derive(Debug)]
pub enum Pattern {
    Any,
    Array(Vec<Pattern>),
    Integer(i64),
    String(&'static str),
    UntypedObject(HashMap<&'static str, Pattern>),
}

impl Pattern {
    pub fn object_from_array<const N: usize>(fields: [(&'static str, Pattern); N]) -> Pattern {
        Pattern::UntypedObject(fields.into_iter().collect())
    }
}

pub fn assert_matches(value: Value, pattern: Pattern) {
    match (value, pattern) {
        (_, Pattern::Any) => {}
        (Value::Array(v), Pattern::Array(p)) => assert_array_matches(v, p),
        (Value::Number(v), Pattern::Integer(p)) => assert_number_matches(v, p),
        (Value::Object(v), Pattern::UntypedObject(p)) => assert_untyped_object_matches(v, p),
        (Value::String(v), Pattern::String(p)) => assert_string_matches(v, p),
        _ => panic!("Values don't have the same type"),
    }
}

fn assert_number_matches(value: Number, pattern: i64) {
    let json_number = value.as_i64().expect("Failed to convert Number to i64");
    assert_eq!(json_number, pattern);
}

fn assert_array_matches(value: Vec<Value>, pattern: Vec<Pattern>) {
    assert_eq!(
        value.len(),
        pattern.len(),
        "Arrays don't have the same length"
    );

    value
        .into_iter()
        .zip(pattern)
        .for_each(|(v, p)| assert_matches(v, p))
}

fn assert_string_matches(value: String, pattern: &'static str) {
    assert_eq!(value, pattern);
}

fn assert_untyped_object_matches(
    mut value: Map<String, Value>,
    pattern: HashMap<&'static str, Pattern>,
) {
    for value_key in value.keys() {
        assert!(
            pattern.contains_key(value_key.as_str()),
            "Field `{}` is included in the object but not matched in the pattern",
            value_key
        );
    }

    for (field, associated_pattern) in pattern {
        let corresponding_value = value.remove(field);

        assert!(
            corresponding_value.is_some(),
            "Field `{}` is matched in the pattern but not found in the JSON",
            field
        );

        let corresponding_value = corresponding_value.unwrap();
        assert_matches(corresponding_value, associated_pattern);
    }
}

/// Allows to encode the method chaining that is necessary to get access to the
/// the value we want.
pub trait ValueExt {
    fn to_array(&self) -> &[Value];
    fn to_object(&self) -> &Map<String, Value>;

    fn deserialize<T: DeserializeOwned>(self) -> T;
}

impl ValueExt for Value {
    fn to_array(&self) -> &[Value] {
        match self {
            Value::Array(array) => array,
            _ => unreachable!(),
        }
    }

    fn to_object(&self) -> &Map<String, Value> {
        match self {
            Value::Object(map) => map,
            _ => unreachable!(),
        }
    }

    fn deserialize<T: DeserializeOwned>(self) -> T {
        serde_json::from_value(self).unwrap()
    }
}

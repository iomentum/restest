use serde_json::{Number, Value};

pub enum Pattern {
    Array(Vec<Pattern>),
    Integer(isize),
}

pub fn assert_matches(value: Value, pattern: Pattern) {
    match (value, pattern) {
        (Value::Number(v), Pattern::Integer(p)) => assert_number_matches(v, p),
        (Value::Array(v), Pattern::Array(p)) => assert_array_matches(v, p),
        _ => panic!("Values don't have the same type"),
    }
}

fn assert_number_matches(value: Number, pattern: isize) {
    let json_number = value.as_i64().expect("Failed to convert Number to i64");
    assert_eq!(json_number as isize, pattern);
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

#[cfg(test)]
use crate::rustifier::*;

#[test]
fn parses_empty_object() {
    let actual = loads("{}").unwrap();
    let expected = JSONElement::Object(HashMap::new());
    assert_eq!(actual, expected);
}

#[test]
fn parses_object_with_elements() {
    let actual = loads(
        r#"{"apple": 1,
    "bear": "2",
    "cat": [1,2,"3"],
    "dog": null,
    "quail": {"sing": "song", "duane": 244}, 
    "drew": false
    }"#,
    )
    .unwrap();
    let expected = JSONElement::Object(
        HashMap::from([
            ("apple", JSONElement::Number(1)),
            ("bear", JSONElement::String(String::from("2"))),
            (
                "cat",
                JSONElement::Array(vec![
                    JSONElement::Number(1),
                    JSONElement::Number(2),
                    JSONElement::String(String::from("3")),
                ]),
            ),
            ("dog", JSONElement::Null),
            (
                "quail",
                JSONElement::Object(HashMap::from([
                    (
                        String::from("sing"),
                        JSONElement::String(String::from("song")),
                    ),
                    (String::from("duane"), JSONElement::Number(244)),
                ])),
            ),
            ("drew", JSONElement::Boolean(false)),
        ])
        .drain()
        .map(|(k, v)| (k.to_owned(), v))
        .collect(),
    );
    assert_eq!(actual, expected);
}

#[test]
fn parses_empty_array() {
    let actual = loads("[]").unwrap();
    let expected = JSONElement::Array(vec![]);
    assert_eq!(actual, expected);
}

#[test]
fn parses_array() {
    let actual = loads(
        r#"[
    "Dog",
    2, 
    false, 
    ["frank"], 
    {"sing": 55},
    null,
    ]"#,
    )
    .unwrap();
    let expected = JSONElement::Array(vec![
        JSONElement::String(String::from("Dog")),
        JSONElement::Number(2),
        JSONElement::Boolean(false),
        JSONElement::Array(vec![JSONElement::String(String::from("frank"))]),
        JSONElement::Object(HashMap::from([(
            "sing".to_string(),
            JSONElement::Number(55),
        )])),
        JSONElement::Null,
    ]);
    assert_eq!(actual, expected);
}

#[test]
fn parses_number() {
    let actual = loads("234").unwrap();
    let expected = JSONElement::Number(234);
    assert_eq!(actual, expected);
}

#[test]
fn parses_string() {
    let actual = loads("\"apple\"").unwrap();
    let expected = JSONElement::String(String::from("apple"));
    assert_eq!(actual, expected);
}

#[test]
fn parses_boolean() {
    let actual = loads("false").unwrap();
    let expected = JSONElement::Boolean(false);
    assert_eq!(actual, expected);
}

#[test]
fn parses_null() {
    let actual = loads("null").unwrap();
    let expected = JSONElement::Null;
    assert_eq!(actual, expected);
}

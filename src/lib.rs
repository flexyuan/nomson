use nom::{sequence::tuple, IResult};

pub fn from_json(json: &str) -> Result<Vec<Entry>, String> {
    parse_json(json)
        .map_err(|err| format!("parse error: {}", err))
        .map(|(_, entries)| entries)
}

fn parse_json(x: &str) -> IResult<&str, Vec<Entry>> {
    let result = nom::sequence::tuple((
        nom::character::complete::multispace0,
        nom::character::complete::char('{'),
        nom::multi::separated_list0(nom::character::complete::char(','), parse_entry),
        nom::character::complete::multispace0,
        nom::character::complete::char('}'),
        nom::character::complete::multispace0,
    ))(x);
    result.map(|(rest, (_, _, entries, _, _, _))| (rest, entries))
}

fn parse_entry(x: &str) -> IResult<&str, Entry> {
    nom::sequence::tuple((
        nom::character::complete::multispace0,
        parse_key,
        nom::character::complete::multispace0,
        nom::character::complete::char(':'),
        nom::character::complete::multispace0,
        parse_value,
        nom::character::complete::multispace0,
    ))(x)
    .map(|(rest, (_, key, _, _, _, value, _))| (rest, Entry { key, value }))
}

// Parse string
fn parse_key(x: &str) -> IResult<&str, String> {
    let r = parse_string(x);
    dbg!(x);
    dbg!(r)
}

fn parse_string(x: &str) -> IResult<&str, String> {
    let parse_chars = nom::bytes::complete::escaped(
        nom::bytes::complete::is_not(r#"\""#),
        '\\',
        nom::character::complete::one_of(r#"\"n"#),
    );
    let result = tuple((
        nom::character::complete::char('"'),
        parse_chars,
        nom::character::complete::char('"'),
    ))(x);
    result.map(|(rest, (_, chars, _))| (rest, chars.to_owned()))
}

fn parse_value(x: &str) -> IResult<&str, Value> {
    let x = dbg!(x);
    let result = nom::branch::alt((
        parse_float,
        parse_int,
        nom::combinator::map(parse_string, |s| Value::Text(s)),
        nom::combinator::map(parse_json, |e| Value::Object(e)),
        nom::combinator::map(parse_list, |e| Value::List(e)),
    ))(x);
    dbg!(result)
}

fn parse_int(x: &str) -> IResult<&str, Value> {
    nom::character::complete::digit1(x)
        .map(|(rest, digits)| (rest, digits.parse::<i32>().map(|i| Value::Int(i)).unwrap()))
}

fn parse_float(x: &str) -> IResult<&str, Value> {
    nom::sequence::tuple((
        nom::character::complete::digit1,
        nom::character::complete::char('.'),
        nom::character::complete::digit1,
    ))(x)
    .map(|(rest, (digits1, _, digits2))| {
        (
            rest,
            format!("{}.{}", digits1, digits2)
                .parse::<f64>()
                .map(|f| Value::Float(f))
                .unwrap(),
        )
    })
}

fn parse_list(x: &str) -> IResult<&str, Vec<Value>> {
    let result = nom::sequence::tuple((
        nom::character::complete::multispace0,
        nom::character::complete::char('['),
        nom::character::complete::multispace0,
        nom::multi::separated_list0(
            nom::sequence::tuple((
                nom::character::complete::multispace0,
                nom::character::complete::char(','),
                nom::character::complete::multispace0,
            )),
            parse_value,
        ),
        nom::character::complete::multispace0,
        nom::character::complete::char(']'),
        nom::character::complete::multispace0,
    ))(x);
    result.map(|(rest, (_, _, _, entries, _, _, _))| (rest, entries))
}

#[derive(Debug, PartialEq)]
pub struct Entry {
    key: String,
    value: Value,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Float(f64),
    Text(String),
    Object(Vec<Entry>),
    List(Vec<Value>),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_empty_json() {
        let json = r#"
            {
            }
        "#;
        let from_json = from_json(json);
        let entries = from_json.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn test_parse_simple_json() {
        let json = r#"
            {
                "key1": 1,
                "key2": 2.0
            }
        "#;
        let from_json = from_json(json);
        let entries = from_json.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, "key1");
        assert_eq!(entries[0].value, Value::Int(1));
        assert_eq!(entries[1].key, "key2");
        assert_eq!(entries[1].value, Value::Float(2.0));
    }

    #[test]
    fn test_parse_json() {
        let json = r#"
            {
                "key1": 1,
                "key2": 2.0,
                "key3": "value3",
                "key4": {
                    "key5": 5
                }
            }
        "#;
        let from_json = from_json(json);
        let entries = from_json.unwrap();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, "key1");
        assert_eq!(entries[0].value, Value::Int(1));
        assert_eq!(entries[1].key, "key2");
        assert_eq!(entries[1].value, Value::Float(2.0));
        assert_eq!(entries[2].key, "key3");
        assert_eq!(entries[2].value, Value::Text("value3".to_string()));
        assert_eq!(entries[3].key, "key4");
        assert_eq!(
            entries[3].value,
            Value::Object(vec![Entry {
                key: "key5".to_string(),
                value: Value::Int(5)
            }])
        );
    }

    #[test]
    fn test_parse_complex_json() {
        let input = r#"
        {
            "glossary": {
                "title": "example glossary",
                "GlossDiv": {
                    "title": "S",
                    "GlossList": {
                        "GlossEntry": {
                            "ID": "SGML",
                            "SortAs": "SGML",
                            "GlossTerm": "Standard Generalized Markup Language",
                            "Acronym": "SGML",
                            "Abbrev": "ISO 8879:1986",
                            "GlossDef": {
                                "para": "A meta-markup language, used to create markup languages such as DocBook.",
                                "GlossSeeAlso": ["GML", "XML"]
                            },
                            "GlossSee": "markup"
                        }
                    }
                }
            }
        }
    "#;
        let from_json = from_json(input);
        let entries = from_json.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key, "glossary");
        assert_eq!(
            entries[0].value,
            Value::Object(vec![Entry {
                key: "title".to_string(),
                value: Value::Text("example glossary".to_string())
            }, Entry {
                key: "GlossDiv".to_string(),
                value: Value::Object(vec![Entry {
                    key: "title".to_string(),
                    value: Value::Text("S".to_string())
                }, Entry {
                    key: "GlossList".to_string(),
                    value: Value::Object(vec![Entry {
                        key: "GlossEntry".to_string(),
                        value: Value::Object(vec![
                            Entry {
                                key: "ID".to_string(),
                                value: Value::Text("SGML".to_string())
                            },
                            Entry {
                                key: "SortAs".to_string(),
                                value: Value::Text("SGML".to_string())
                            },
                            Entry {
                                key: "GlossTerm".to_string(),
                                value: Value::Text("Standard Generalized Markup Language".to_string())
                            },
                            Entry {
                                key: "Acronym".to_string(),
                                value: Value::Text("SGML".to_string())
                            },
                            Entry {
                                key: "Abbrev".to_string(),
                                value: Value::Text("ISO 8879:1986".to_string())
                            },
                            Entry {
                                key: "GlossDef".to_string(),
                                value: Value::Object(vec![Entry {
                                    key: "para".to_string(),
                                    value: Value::Text("A meta-markup language, used to create markup languages such as DocBook.".to_string())
                                }, Entry {
                                    key: "GlossSeeAlso".to_string(),
                                    value: Value::List(vec![
                                        Value::Text("GML".to_string()),
                                        Value::Text("XML".to_string())
                                    ])
                                }])
                            },
                            Entry {
                                key: "GlossSee".to_string(),
                                value: Value::Text("markup".to_string())
                            }
                        ])
                    }])
                }])
            }])
        );
    }

    #[test]
    fn test_parse_string() {
        let input = r#""abc""#;
        let result = parse_string(input);
        assert_eq!(result.unwrap(), ("", "abc".to_string()));
    }

    #[test]
    fn test_parse_string_with_escape() {
        let input = r#""a\"bc""#;
        let result = parse_string(input);
        assert_eq!(result.unwrap(), ("", r#"a\"bc"#.to_string()));
    }
}

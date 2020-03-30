use std::string;

use serde_json;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use serde::ser::{Serialize, Serializer};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Query {
    And(Vec<Query>),
    Or(Vec<Query>),
    Not(Box<Query>),
    Eq(String, String),
    Neq(String, String),
    Gt(String, String),
    Gte(String, String),
    Lt(String, String),
    Lte(String, String),
    Like(String, String),
    In(String, Vec<String>),
}

impl Serialize for Query {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
        self.to_value().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Query
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let v = Value::deserialize(deserializer)?;

        match v {
            serde_json::Value::Object(map) => {
                parse_query(map)
                    .map_err(|err| de::Error::missing_field(err))
            }
            serde_json::Value::Array(array) => {
                // cast old restrictions format to wql
                let mut res: Vec<serde_json::Value> = Vec::new();
                for sub_query in array {
                    let sub_query: serde_json::Map<String, serde_json::Value> =
                        sub_query.as_object()
                            .ok_or_else(|| de::Error::custom("Restriction is invalid"))?
                            .clone()
                            .into_iter()
                            .filter(|&(_, ref v)| !v.is_null())
                            .collect();

                    if !sub_query.is_empty() {
                        res.push(serde_json::Value::Object(sub_query));
                    }
                }

                let mut map = serde_json::Map::new();
                map.insert("$or".to_string(), serde_json::Value::Array(res));

                parse_query(map).map_err(|err| de::Error::custom(err))
            }
            _ => Err(de::Error::missing_field("Restriction must be either object or array"))
        }
    }
}

impl Query {
    pub fn optimise(self) -> Option<Query> {
        match self {
            Query::Not(boxed_operator) => if let Query::Not(nested_operator) = *boxed_operator {
                Some(*nested_operator)
            } else {
                Some(Query::Not(boxed_operator))
            },
            Query::And(suboperators)  if suboperators.len() == 0 => {
                None
            }
            Query::And(mut suboperators) if suboperators.len() == 1 => {
                suboperators.remove(0).optimise()
            }
            Query::And(suboperators) => {
                let mut suboperators: Vec<Query> =
                    suboperators
                        .into_iter()
                        .flat_map(|operator| operator.optimise())
                        .collect();

                match suboperators.len() {
                    0 => None,
                    1 => Some(suboperators.remove(0)),
                    _ => Some(Query::And(suboperators)),
                }
            }
            Query::Or(suboperators) if suboperators.len() == 0 => {
                None
            }
            Query::Or(mut suboperators) if suboperators.len() == 1 => {
                suboperators.remove(0).optimise()
            }
            Query::Or(suboperators) => {
                let mut suboperators: Vec<Query> =
                    suboperators
                        .into_iter()
                        .flat_map(|operator| operator.optimise())
                        .collect();

                match suboperators.len() {
                    0 => None,
                    1 => Some(suboperators.remove(0)),
                    _ => Some(Query::Or(suboperators)),
                }
            }
            Query::In(key, mut targets) if targets.len() == 1 => {
                Some(Query::Eq(key, targets.remove(0)))
            }
            Query::In(key, targets) => {
                Some(Query::In(key, targets))
            }
            _ => Some(self)
        }
    }

    fn to_value(&self) -> serde_json::Value {
        match *self {
            Query::Eq(ref tag_name, ref tag_value) => json!({tag_name: tag_value}),
            Query::Neq(ref tag_name, ref tag_value) => json!({tag_name: {"$neq": tag_value}}),
            Query::Gt(ref tag_name, ref tag_value) => json!({tag_name: {"$gt": tag_value}}),
            Query::Gte(ref tag_name, ref tag_value) => json!({tag_name: {"$gte": tag_value}}),
            Query::Lt(ref tag_name, ref tag_value) => json!({tag_name: {"$lt": tag_value}}),
            Query::Lte(ref tag_name, ref tag_value) => json!({tag_name: {"$lte": tag_value}}),
            Query::Like(ref tag_name, ref tag_value) => json!({tag_name: {"$like": tag_value}}),
            Query::In(ref tag_name, ref tag_values) => json!({tag_name: {"$in": tag_values}}),
            Query::And(ref operators) => {
                if !operators.is_empty() {
                    json!({
                        "$and": operators.iter().map(|q: &Query| q.to_value()).collect::<Vec<serde_json::Value>>()
                    })
                } else {
                    json!({})
                }
            }
            Query::Or(ref operators) => {
                if !operators.is_empty() {
                    json!({
                        "$or": operators.iter().map(|q: &Query| q.to_value()).collect::<Vec<serde_json::Value>>()
                    })
                } else {
                    json!({})
                }
            }
            Query::Not(ref stmt) => json!({"$not": stmt.to_value()}),
        }
    }
}

impl Default for Query {
    fn default() -> Self {
        Query::And(Vec::new())
    }
}

impl string::ToString for Query {
    fn to_string(&self) -> String {
        self.to_value().to_string()
    }
}

fn parse_query(map: serde_json::Map<String, serde_json::Value>) -> Result<Query, &'static str> {
    let mut operators: Vec<Query> = Vec::new();

    for (key, value) in map {
        if let Some(operator_) = parse_operator(key, value)? {
            operators.push(operator_);
        }
    }

    let query = if operators.len() == 1 {
        operators.remove(0)
    } else {
        Query::And(operators)
    };

    Ok(query)
}

fn parse_operator(key: String, value: serde_json::Value) -> Result<Option<Query>, &'static str> {
    match (key.as_str(), value) {
        ("$and", serde_json::Value::Array(values)) if values.is_empty() => Ok(None),
        ("$and", serde_json::Value::Array(values)) => {
            let operators: Vec<Query> = parse_list_operators(values)?;
            Ok(Some(Query::And(operators)))
        }
        ("$and", _) => Err("$and must be array of JSON objects"),
        ("$or", serde_json::Value::Array(values)) if values.is_empty() => Ok(None),
        ("$or", serde_json::Value::Array(values)) => {
            let operators: Vec<Query> = parse_list_operators(values)?;
            Ok(Some(Query::Or(operators)))
        }
        ("$or", _) => Err("$or must be array of JSON objects"),
        ("$not", serde_json::Value::Object(map)) => {
            let operator = parse_query(map)?;
            Ok(Some(Query::Not(Box::new(operator))))
        }
        ("$not", _) => Err("$not must be JSON object"),
        (_, serde_json::Value::String(value)) => {
            Ok(Some(Query::Eq(key, value)))
        }
        (_, serde_json::Value::Object(map)) => {
            if map.len() == 1 {
                let (operator_name, value) = map.into_iter().next().unwrap();
                parse_single_operator(operator_name, key, value)
                    .map(|operator| Some(operator))
            } else {
                Err("value must be JSON object of length 1")
            }
        }
        (_, _) => Err("Unsupported value")
    }
}

fn parse_list_operators(operators: Vec<serde_json::Value>) -> Result<Vec<Query>, &'static str> {
    let mut out_operators: Vec<Query> = Vec::with_capacity(operators.len());

    for value in operators.into_iter() {
        if let serde_json::Value::Object(map) = value {
            let suboperator = parse_query(map)?;
            out_operators.push(suboperator);
        } else {
            return Err("operator must be array of JSON objects");
        }
    }

    Ok(out_operators)
}

fn parse_single_operator(operator_name: String, key: String, value: serde_json::Value) -> Result<Query, &'static str> {
    match (&*operator_name, value) {
        ("$neq", serde_json::Value::String(value_)) => Ok(Query::Neq(key, value_)),
        ("$neq", _) => Err("$neq must be used with string"),
        ("$gt", serde_json::Value::String(value_)) => Ok(Query::Gt(key, value_)),
        ("$gt", _) => Err("$gt must be used with string"),
        ("$gte", serde_json::Value::String(value_)) => Ok(Query::Gte(key, value_)),
        ("$gte", _) => Err("$gte must be used with string"),
        ("$lt", serde_json::Value::String(value_)) => Ok(Query::Lt(key, value_)),
        ("$lt", _) => Err("$lt must be used with string"),
        ("$lte", serde_json::Value::String(value_)) => Ok(Query::Lte(key, value_)),
        ("$lte", _) => Err("$lte must be used with string"),
        ("$like", serde_json::Value::String(value_)) => Ok(Query::Like(key, value_)),
        ("$like", _) => Err("$like must be used with string"),
        ("$in", serde_json::Value::Array(values)) => {
            let mut target_values: Vec<String> = Vec::with_capacity(values.len());

            for v in values.into_iter() {
                if let serde_json::Value::String(s) = v {
                    target_values.push(s);
                } else {
                    return Err("$in must be used with array of strings");
                }
            }

            Ok(Query::In(key, target_values))
        }
        ("$in", _) => Err("$in must be used with array of strings"),
        (_, _) => Err("Unknown operator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;

    fn _random_string(len: usize) -> String {
        thread_rng().sample_iter(&Alphanumeric).take(len).collect()
    }

    /// parse
    #[test]
    fn test_simple_operator_empty_json_parse() {
        let json = "{}";

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_explicit_empty_and_parse() {
        let json = r#"{"$and":[]}"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_empty_or_parse() {
        let json = r#"{"$or":[]}"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_empty_not_parse() {
        let json = r#"{"$not":{}}"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        let expected = Query::Not(Box::new(Query::And(vec![])));

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_eq_plaintext_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":"{}"}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Eq(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$neq":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Neq(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$gt":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Gt(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$gte":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Gte(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$lt":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Lt(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_lte_plaintext_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$lte":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Lte(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$like":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Like(name1, value1);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_in_plaintext_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$in":["{}"]}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::In(name1, vec![value1]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_simple_operator_in_plaintexts_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let value2 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"{}":{{"$in":["{}","{}","{}"]}}}}"#, name1, value1, value2, value3);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::In(name1, vec![value1, value2, value3]);

        assert_eq!(query, expected);
    }


    #[test]
    fn test_and_with_one_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":"{}"}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Eq(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Neq(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Gt(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Gte(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Lt(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Lte(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Like(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::In(name1, vec![value1])
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_one_not_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Not(
                    Box::new(
                        Query::Eq(name1, value1)
                    )
                )
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    #[ignore] // order
    fn test_short_and_with_multiple_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"{}":"{}","{}":"{}","{}":"{}"}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Eq(name1, value1),
                Query::Eq(name2, value2),
                Query::Eq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Eq(name1, value1),
                Query::Eq(name2, value2),
                Query::Eq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Neq(name1, value1),
                Query::Neq(name2, value2),
                Query::Neq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Gt(name1, value1),
                Query::Gt(name2, value2),
                Query::Gt(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Gte(name1, value1),
                Query::Gte(name2, value2),
                Query::Gte(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Lt(name1, value1),
                Query::Lt(name2, value2),
                Query::Lt(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Lte(name1, value1),
                Query::Lte(name2, value2),
                Query::Lte(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Like(name1, value1),
                Query::Like(name2, value2),
                Query::Like(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::In(name1, vec![value1]),
                Query::In(name2, vec![value2]),
                Query::In(name3, vec![value3])
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_not_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Not(
                    Box::new(
                        Query::Eq(name1, value1)
                    )
                ),
                Query::Not(
                    Box::new(
                        Query::Eq(name2, value2)
                    )
                ),
                Query::Not(
                    Box::new(
                        Query::Eq(name3, value3)
                    )
                ),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_with_multiple_mixed_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8a = _random_string(10);
        let value8b = _random_string(10);
        let name9 = _random_string(10);
        let value9 = _random_string(10);

        let json = format!(r#"{{"$and":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
                           name4, value4,
                           name5, value5,
                           name6, value6,
                           name7, value7,
                           name8, value8a, value8b,
                           name9, value9,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(
            vec![
                Query::Eq(name1, value1),
                Query::Neq(name2, value2),
                Query::Gt(name3, value3),
                Query::Gte(name4, value4),
                Query::Lt(name5, value5),
                Query::Lte(name6, value6),
                Query::Like(name7, value7),
                Query::In(name8, vec![value8a, value8b]),
                Query::Not(
                    Box::new(
                        Query::Eq(name9, value9)
                    )
                ),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":"{}"}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Eq(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Neq(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Gt(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Gte(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Lt(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Lte(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Like(name1, value1)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::In(name1, vec![value1])
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_one_not_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Not(
                    Box::new(
                        Query::Eq(name1, value1)
                    )
                )
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Eq(name1, value1),
                Query::Eq(name2, value2),
                Query::Eq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Neq(name1, value1),
                Query::Neq(name2, value2),
                Query::Neq(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Gt(name1, value1),
                Query::Gt(name2, value2),
                Query::Gt(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Gte(name1, value1),
                Query::Gte(name2, value2),
                Query::Gte(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Lt(name1, value1),
                Query::Lt(name2, value2),
                Query::Lt(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Lte(name1, value1),
                Query::Lte(name2, value2),
                Query::Lte(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Like(name1, value1),
                Query::Like(name2, value2),
                Query::Like(name3, value3)
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::In(name1, vec![value1]),
                Query::In(name2, vec![value2]),
                Query::In(name3, vec![value3]),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_not_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Not(
                    Box::new(
                        Query::Eq(name1, value1)
                    )
                ),
                Query::Not(
                    Box::new(
                        Query::Eq(name2, value2)
                    )
                ),
                Query::Not(
                    Box::new(
                        Query::Eq(name3, value3)
                    )
                ),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_or_with_multiple_mixed_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8a = _random_string(10);
        let value8b = _random_string(10);
        let name9 = _random_string(10);
        let value9 = _random_string(10);

        let json = format!(r#"{{"$or":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
                           name4, value4,
                           name5, value5,
                           name6, value6,
                           name7, value7,
                           name8, value8a, value8b,
                           name9, value9,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Eq(name1, value1),
                Query::Neq(name2, value2),
                Query::Gt(name3, value3),
                Query::Gte(name4, value4),
                Query::Lt(name5, value5),
                Query::Lte(name6, value6),
                Query::Like(name7, value7),
                Query::In(name8, vec![value8a, value8b]),
                Query::Not(
                    Box::new(
                        Query::Eq(name9, value9)
                    )
                ),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_eq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":"{}"}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::Eq(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_neq_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$neq":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::Neq(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_gt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$gt":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::Gt(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_gte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$gte":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::Gte(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_lt_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$lt":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::Lt(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_lte_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$lte":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::Lte(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_like_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$like":"{}"}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::Like(name1, value1)
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_not_with_one_in_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let json = format!(r#"{{"$not":{{"{}":{{"$in":["{}"]}}}}}}"#, name1, value1);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::In(name1, vec![value1])
            )
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_and_or_not_complex_case_parse() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8 = _random_string(10);

        let json = format!(r#"{{"$not":{{"$and":[{{"{}":"{}"}},{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"$not":{{"{}":{{"$lte":"{}"}}}}}},{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"$not":{{"{}":{{"$gte":"{}"}}}}}}]}}]}},{{"$not":{{"{}":{{"$like":"{}"}}}}}},{{"$and":[{{"{}":"{}"}},{{"$not":{{"{}":{{"$neq":"{}"}}}}}}]}}]}}}}"#,
                           name1, value1,
                           name2, value2,
                           name3, value3,
                           name4, value4,
                           name5, value5,
                           name6, value6,
                           name7, value7,
                           name8, value8,
        );

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Not(
            Box::new(
                Query::And(
                    vec![
                        Query::Eq(name1, value1),
                        Query::Or(
                            vec![
                                Query::Gt(name2, value2),
                                Query::Not(
                                    Box::new(
                                        Query::Lte(name3, value3)
                                    )
                                ),
                                Query::And(
                                    vec![
                                        Query::Lt(name4, value4),
                                        Query::Not(
                                            Box::new(
                                                Query::Gte(name5, value5)
                                            )
                                        ),
                                    ]
                                )
                            ]
                        ),
                        Query::Not(
                            Box::new(
                                Query::Like(name6, value6)
                            )
                        ),
                        Query::And(
                            vec![
                                Query::Eq(name7, value7),
                                Query::Not(
                                    Box::new(
                                        Query::Neq(name8, value8)
                                    )
                                ),
                            ]
                        )
                    ]
                )
            )
        );

        assert_eq!(query, expected);
    }

    /// to string
    #[test]
    fn test_simple_operator_empty_and_to_string() {
        let query = Query::And(vec![]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = "{}";

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_empty_or_to_string() {
        let query = Query::Or(vec![]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = "{}";

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_empty_not_to_string() {
        let query = Query::Not(Box::new(Query::And(vec![])));

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = r#"{"$not":{}}"#;

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Eq(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":"{}"}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Neq(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$neq":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }


    #[test]
    fn test_simple_operator_gt_plaintext_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Gt(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$gt":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Gte(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$gte":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Lt(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$lt":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Lte(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$lte":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Like(name1.clone(), value1.clone());

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$like":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::In(name1.clone(), vec![value1.clone()]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$in":["{}"]}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_simple_operator_in_multimply_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let value2 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::In(name1.clone(), vec![value1.clone(), value2.clone(), value3.clone()]);

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"{}":{{"$in":["{}","{}","{}"]}}}}"#, name1, value1, value2, value3);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Eq(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":"{}"}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Neq(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Gt(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Gte(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Lt(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Lte(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Like(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::In(name1.clone(), vec![value1.clone()])
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_one_not_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Not(
                    Box::new(
                        Query::Eq(name1.clone(), value1.clone())
                    )
                )
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Eq(name1.clone(), value1.clone()),
                Query::Eq(name2.clone(), value2.clone()),
                Query::Eq(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Neq(name1.clone(), value1.clone()),
                Query::Neq(name2.clone(), value2.clone()),
                Query::Neq(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Gt(name1.clone(), value1.clone()),
                Query::Gt(name2.clone(), value2.clone()),
                Query::Gt(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Gte(name1.clone(), value1.clone()),
                Query::Gte(name2.clone(), value2.clone()),
                Query::Gte(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Lt(name1.clone(), value1.clone()),
                Query::Lt(name2.clone(), value2.clone()),
                Query::Lt(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Lte(name1.clone(), value1.clone()),
                Query::Lte(name2.clone(), value2.clone()),
                Query::Lte(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Like(name1.clone(), value1.clone()),
                Query::Like(name2.clone(), value2.clone()),
                Query::Like(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::In(name1.clone(), vec![value1.clone()]),
                Query::In(name2.clone(), vec![value2.clone()]),
                Query::In(name3.clone(), vec![value3.clone()])
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_not_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Not(
                    Box::new(
                        Query::Eq(name1.clone(), value1.clone())
                    )
                ),
                Query::Not(
                    Box::new(
                        Query::Eq(name2.clone(), value2.clone())
                    )
                ),
                Query::Not(
                    Box::new(
                        Query::Eq(name3.clone(), value3.clone())
                    )
                ),
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_with_multiple_mixed_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8a = _random_string(10);
        let value8b = _random_string(10);
        let name9 = _random_string(10);
        let value9 = _random_string(10);

        let query = Query::And(
            vec![
                Query::Eq(name1.clone(), value1.clone()),
                Query::Neq(name2.clone(), value2.clone()),
                Query::Gt(name3.clone(), value3.clone()),
                Query::Gte(name4.clone(), value4.clone()),
                Query::Lt(name5.clone(), value5.clone()),
                Query::Lte(name6.clone(), value6.clone()),
                Query::Like(name7.clone(), value7.clone()),
                Query::In(name8.clone(), vec![value8a.clone(), value8b.clone()]),
                Query::Not(
                    Box::new(
                        Query::Eq(name9.clone(), value9.clone())
                    )
                ),
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$and":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
                               name4, value4,
                               name5, value5,
                               name6, value6,
                               name7, value7,
                               name8, value8a, value8b,
                               name9, value9,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Eq(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":"{}"}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Neq(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Gt(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}}]}}"#, name1, value1);
        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Gte(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Lt(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Lte(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Like(name1.clone(), value1.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$like":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::In(name1.clone(), vec![value1.clone()])
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_one_not_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Not(
                    Box::new(
                        Query::Eq(name1.clone(), value1.clone())
                    )
                )
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"$not":{{"{}":"{}"}}}}]}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Eq(name1.clone(), value1.clone()),
                Query::Eq(name2.clone(), value2.clone()),
                Query::Eq(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":"{}"}},{{"{}":"{}"}},{{"{}":"{}"}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Neq(name1.clone(), value1.clone()),
                Query::Neq(name2.clone(), value2.clone()),
                Query::Neq(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$neq":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Gt(name1.clone(), value1.clone()),
                Query::Gt(name2.clone(), value2.clone()),
                Query::Gt(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gt":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Gte(name1.clone(), value1.clone()),
                Query::Gte(name2.clone(), value2.clone()),
                Query::Gte(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$gte":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Lt(name1.clone(), value1.clone()),
                Query::Lt(name2.clone(), value2.clone()),
                Query::Lt(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lt":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Lte(name1.clone(), value1.clone()),
                Query::Lte(name2.clone(), value2.clone()),
                Query::Lte(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$lte":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Like(name1.clone(), value1.clone()),
                Query::Like(name2.clone(), value2.clone()),
                Query::Like(name3.clone(), value3.clone())
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$like":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::In(name1.clone(), vec![value1.clone()]),
                Query::In(name2.clone(), vec![value2.clone()]),
                Query::In(name3.clone(), vec![value3.clone()]),
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}},{{"{}":{{"$in":["{}"]}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_not_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Not(
                    Box::new(
                        Query::Eq(name1.clone(), value1.clone())
                    )
                ),
                Query::Not(
                    Box::new(
                        Query::Eq(name2.clone(), value2.clone())
                    )
                ),
                Query::Not(
                    Box::new(
                        Query::Eq(name3.clone(), value3.clone())
                    )
                ),
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_or_with_multiple_mixed_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8a = _random_string(10);
        let value8b = _random_string(10);
        let name9 = _random_string(10);
        let value9 = _random_string(10);

        let query = Query::Or(
            vec![
                Query::Eq(name1.clone(), value1.clone()),
                Query::Neq(name2.clone(), value2.clone()),
                Query::Gt(name3.clone(), value3.clone()),
                Query::Gte(name4.clone(), value4.clone()),
                Query::Lt(name5.clone(), value5.clone()),
                Query::Lte(name6.clone(), value6.clone()),
                Query::Like(name7.clone(), value7.clone()),
                Query::In(name8.clone(), vec![value8a.clone(), value8b.clone()]),
                Query::Not(
                    Box::new(
                        Query::Eq(name9.clone(), value9.clone())
                    )
                ),
            ]
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$or":[{{"{}":"{}"}},{{"{}":{{"$neq":"{}"}}}},{{"{}":{{"$gt":"{}"}}}},{{"{}":{{"$gte":"{}"}}}},{{"{}":{{"$lt":"{}"}}}},{{"{}":{{"$lte":"{}"}}}},{{"{}":{{"$like":"{}"}}}},{{"{}":{{"$in":["{}","{}"]}}}},{{"$not":{{"{}":"{}"}}}}]}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
                               name4, value4,
                               name5, value5,
                               name6, value6,
                               name7, value7,
                               name8, value8a, value8b,
                               name9, value9,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_eq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::Eq(name1.clone(), value1.clone())
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":"{}"}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_neq_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::Neq(name1.clone(), value1.clone())
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$neq":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_gt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::Gt(name1.clone(), value1.clone())
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$gt":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_gte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::Gte(name1.clone(), value1.clone())
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$gte":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_lt_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::Lt(name1.clone(), value1.clone())
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$lt":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_lte_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::Lte(name1.clone(), value1.clone())
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$lte":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_like_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::Like(name1.clone(), value1.clone())
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$like":"{}"}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_not_with_one_in_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::In(name1.clone(), vec![value1.clone()])
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"{}":{{"$in":["{}"]}}}}}}"#, name1, value1);

        assert_eq!(json, expected);
    }

    #[test]
    fn test_and_or_not_complex_case_to_string() {
        let name1 = _random_string(10);
        let value1 = _random_string(10);
        let name2 = _random_string(10);
        let value2 = _random_string(10);
        let name3 = _random_string(10);
        let value3 = _random_string(10);
        let name4 = _random_string(10);
        let value4 = _random_string(10);
        let name5 = _random_string(10);
        let value5 = _random_string(10);
        let name6 = _random_string(10);
        let value6 = _random_string(10);
        let name7 = _random_string(10);
        let value7 = _random_string(10);
        let name8 = _random_string(10);
        let value8 = _random_string(10);

        let query = Query::Not(
            Box::new(
                Query::And(
                    vec![
                        Query::Eq(name1.clone(), value1.clone()),
                        Query::Or(
                            vec![
                                Query::Gt(name2.clone(), value2.clone()),
                                Query::Not(
                                    Box::new(
                                        Query::Lte(name3.clone(), value3.clone())
                                    )
                                ),
                                Query::And(
                                    vec![
                                        Query::Lt(name4.clone(), value4.clone()),
                                        Query::Not(
                                            Box::new(
                                                Query::Gte(name5.clone(), value5.clone())
                                            )
                                        ),
                                    ]
                                )
                            ]
                        ),
                        Query::Not(
                            Box::new(
                                Query::Like(name6.clone(), value6.clone())
                            )
                        ),
                        Query::And(
                            vec![
                                Query::Eq(name7.clone(), value7.clone()),
                                Query::Not(
                                    Box::new(
                                        Query::Neq(name8.clone(), value8.clone())
                                    )
                                ),
                            ]
                        )
                    ]
                )
            )
        );

        let json = ::serde_json::to_string(&query).unwrap();

        let expected = format!(r#"{{"$not":{{"$and":[{{"{}":"{}"}},{{"$or":[{{"{}":{{"$gt":"{}"}}}},{{"$not":{{"{}":{{"$lte":"{}"}}}}}},{{"$and":[{{"{}":{{"$lt":"{}"}}}},{{"$not":{{"{}":{{"$gte":"{}"}}}}}}]}}]}},{{"$not":{{"{}":{{"$like":"{}"}}}}}},{{"$and":[{{"{}":"{}"}},{{"$not":{{"{}":{{"$neq":"{}"}}}}}}]}}]}}}}"#,
                               name1, value1,
                               name2, value2,
                               name3, value3,
                               name4, value4,
                               name5, value5,
                               name6, value6,
                               name7, value7,
                               name8, value8,
        );

        assert_eq!(json, expected);
    }

    #[test]
    fn test_old_format() {
        let name1 = _random_string(10);
        let name2 = _random_string(10);
        let value1 = _random_string(10);
        let value2 = _random_string(10);

        let json = format!(r#"[{{"{}":"{}"}}, {{"{}":"{}"}}]"#, name1, value1, name2, value2);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(
            vec![
                Query::Eq(name1, value1),
                Query::Eq(name2, value2),
            ]
        );

        assert_eq!(query, expected);
    }

    #[test]
    fn test_old_format_empty() {
        let json = format!(r#"[]"#);

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::And(vec![]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_old_format_with_nulls() {
        let name1 = _random_string(10);
        let name2 = _random_string(10);
        let value1 = _random_string(10);

        let json = json!(vec ! [json ! ({name1.clone(): value1.clone()}), json ! ({name2.clone(): serde_json::Value::Null})]).to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        let expected = Query::Or(vec![
            Query::Eq(name1, value1)
        ]);

        assert_eq!(query, expected);
    }

    #[test]
    fn test_optimise_and() {
        let json = r#"{}"#;

        let query: Query = ::serde_json::from_str(json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_or() {
        let json = r#"[]"#;

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_single_nested_and() {
        let json = json!({
            "$and": [
                {
                    "$and": []
                }
            ]
        }).to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_several_nested_and() {
        let json = json!({
            "$and": [
                {
                    "$and": []
                },
                {
                    "$and": []
                }
            ]
        }).to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_single_nested_or() {
        let json = json!({
            "$and": [
                {
                    "$or": []
                }
            ]
        }).to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }

    #[test]
    fn test_optimise_several_nested_or() {
        let json = json!({
            "$and": [
                {
                    "$or": []
                },
                {
                    "$or": []
                }
            ]
        }).to_string();

        let query: Query = ::serde_json::from_str(&json).unwrap();

        assert_eq!(query.optimise(), None);
    }
}


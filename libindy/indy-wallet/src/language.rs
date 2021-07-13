use std::string;

use indy_api_types::errors::prelude::*;

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum TagName {
    EncryptedTagName(Vec<u8>),
    PlainTagName(Vec<u8>),
}

impl TagName {
    pub fn from(s: String) -> IndyResult<TagName> {
        if s.is_empty() || s.starts_with('~') && s.len() == 1 {
            return Err(err_msg(IndyErrorKind::WalletQueryError, "Tag name must not be empty"));
        }

        if s.starts_with('~') {
            Ok(TagName::PlainTagName(s.into_bytes()[1..].to_vec()))
        } else {
            Ok(TagName::EncryptedTagName(s.into_bytes()))
        }
    }

}

impl string::ToString for TagName {
    fn to_string(&self) -> String {
        match *self {
            TagName::EncryptedTagName(ref v) => format!(r#""{}""#, base64::encode(v)),
            TagName::PlainTagName(ref v) => format!(r#""~{}""#, base64::encode(v))
        }
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum TargetValue {
    Unencrypted(String),
    Encrypted(Vec<u8>),
}

impl From<String> for TargetValue {
    fn from(s: String) -> TargetValue {
        TargetValue::Unencrypted(s)
    }
}

impl string::ToString for TargetValue {
    fn to_string(&self) -> String {
        match *self {
            TargetValue::Unencrypted(ref s) => format!(r#""{}""#, s),
            TargetValue::Encrypted(ref v) => format!(r#""{}""#, base64::encode(v)),
        }
    }
}

#[derive(Debug, Hash, Clone)]
pub enum Operator {
    And(Vec<Operator>),
    Or(Vec<Operator>),
    Not(Box<Operator>),
    Eq(TagName, TargetValue),
    Neq(TagName, TargetValue),
    Gt(TagName, TargetValue),
    Gte(TagName, TargetValue),
    Lt(TagName, TargetValue),
    Lte(TagName, TargetValue),
    Like(TagName, TargetValue),
    In(TagName, Vec<TargetValue>),
}

impl string::ToString for Operator {
    fn to_string(&self) -> String {
        match *self {
            Operator::Eq(ref tag_name, ref tag_value) => format!(r#"{{{}:{}}}"#, tag_name.to_string(), tag_value.to_string()),
            Operator::Neq(ref tag_name, ref tag_value) => format!(r#"{{{}:{{"$neq":{}}}}}"#, tag_name.to_string(), tag_value.to_string()),
            Operator::Gt(ref tag_name, ref tag_value) => format!(r#"{{{}:{{"$gt":{}}}}}"#, tag_name.to_string(), tag_value.to_string()),
            Operator::Gte(ref tag_name, ref tag_value) => format!(r#"{{{}:{{"$gte":{}}}}}"#, tag_name.to_string(), tag_value.to_string()),
            Operator::Lt(ref tag_name, ref tag_value) => format!(r#"{{{}:{{"$lt":{}}}}}"#, tag_name.to_string(), tag_value.to_string()),
            Operator::Lte(ref tag_name, ref tag_value) => format!(r#"{{{}:{{"$lte":{}}}}}"#, tag_name.to_string(), tag_value.to_string()),
            Operator::Like(ref tag_name, ref tag_value) => format!(r#"{{{}:{{"$like":{}}}}}"#, tag_name.to_string(), tag_value.to_string()),
            Operator::In(ref tag_name, ref tag_values) => {
                format!(
                    r#"{{{}:{{"$in":[{}]}}}}"#,
                    tag_name.to_string(),
                    tag_values.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(",")
                )
            }
            Operator::And(ref operators) => {
                if !operators.is_empty() {
                    format!(
                        r#"{{"$and":[{}]}}"#,
                        operators.iter().map(|o: &Operator| { o.to_string() }).collect::<Vec<String>>().join(","))
                } else { "{}".to_string() }
            },
            Operator::Or(ref operators) => {
                if !operators.is_empty() {
                    format!(
                        r#"{{"$or":[{}]}}"#,
                        operators.iter().map(|o: &Operator| { o.to_string() }).collect::<Vec<String>>().join(","))
                } else { "{}".to_string() }
            },
            Operator::Not(ref stmt) => format!(r#"{{"$not":{}}}"#, stmt.to_string()),
        }
    }
}


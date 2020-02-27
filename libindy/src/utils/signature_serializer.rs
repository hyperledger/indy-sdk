use indy_api_types::errors::prelude::*;
use serde_json::Value;
use indy_utils::crypto::hash::Hash;
use crate::domain::ledger::constants::{ATTRIB, GET_ATTR};

pub fn serialize_signature(v: Value) -> Result<String, IndyError> {
    let _type = v["operation"]["type"].clone();
    _serialize_signature(v, true, _type.as_str())
}

fn _serialize_signature(v: Value, is_top_level: bool, _type: Option<&str>) -> Result<String, IndyError> {
    match v {
        Value::Bool(value) => Ok(if value { "True".to_string() } else { "False".to_string() }),
        Value::Number(value) => Ok(value.to_string()),
        Value::String(value) => Ok(value),
        Value::Array(array) => {
            array
                .into_iter()
                .map(|element| _serialize_signature(element, false, _type))
                .collect::<Result<Vec<String>, IndyError>>()
                .map(|res| res.join(","))
        }
        Value::Object(map) => {
            let mut result = "".to_string();
            let mut in_middle = false;
            for key in map.keys() {
                // Skip signature field at top level as in python code
                if is_top_level && (key == "signature" || key == "fees" || key == "signatures") { continue; }

                if in_middle {
                    result += "|";
                }

                let mut value = map[key].clone();
                if (_type == Some(ATTRIB) || _type == Some(GET_ATTR)) && (key == "raw" || key == "hash" || key == "enc") {
                    // do it only for attribute related request
                    let mut ctx = Hash::new_context()?;

                    ctx.update(&value
                        .as_str()
                        .ok_or_else(|| IndyError::from_msg(IndyErrorKind::InvalidState, "Cannot update hash context"))?
                        .as_bytes())?;

                    value = Value::String(hex::encode(ctx.finish()?.as_ref()));
                }
                result = result + key + ":" + &_serialize_signature(value, false, _type)?;
                in_middle = true;
            }
            Ok(result)
        }
        _ => Ok("".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_serialize_works() {
        let data = r#"{
                        "name": "John Doe",
                        "age": 43,
                        "operation": {
                            "dest": 54
                        },
                        "phones": [
                          "1234567",
                          "2345678",
                          {"rust": 5, "age": 1},
                          3
                        ]
                    }"#;
        let msg: Value = serde_json::from_str(data).unwrap();

        let result = "age:43|name:John Doe|operation:dest:54|phones:1234567,2345678,age:1|rust:5,3";

        assert_eq!(serialize_signature(msg).unwrap(), result)
    }


    #[test]
    fn signature_serialize_works_for_skipped_fields() {
        let data = r#"{
                        "name": "John Doe",
                        "age": 43,
                        "operation": {
                            "type": "100",
                            "hash": "cool hash",
                            "dest": 54
                        },
			"fees": "fees1",
			"signature": "sign1",
			"signatures": "sign-m",
                        "phones": [
                          "1234567",
                          "2345678",
                          {"rust": 5, "age": 1},
                          3
                        ]
                    }"#;
        let msg: Value = serde_json::from_str(data).unwrap();

        let result = "age:43|name:John Doe|operation:dest:54|hash:46aa0c92129b33ee72ee1478d2ae62fa6e756869dedc6c858af3214a6fcf1904|type:100|phones:1234567,2345678,age:1|rust:5,3";

        assert_eq!(serialize_signature(msg).unwrap(), result)
    }


    #[test]
    fn signature_serialize_works_with_raw_hash_for_attrib_related_type() {
        let data = r#"{
                        "name": "John Doe",
                        "age": 43,
                        "operation": {
                            "type": "100",
                            "hash": "cool hash",
                            "dest": 54,
                            "raw": "string for hash"
                        },
                        "phones": [
                          "1234567",
                          "2345678",
                          {"rust": 5, "age": 1},
                          3
                        ]
                    }"#;
        let msg: Value = serde_json::from_str(data).unwrap();

        let result = "age:43|name:John Doe|operation:dest:54|hash:46aa0c92129b33ee72ee1478d2ae62fa6e756869dedc6c858af3214a6fcf1904|raw:1dcd0759ce38f57049344a6b3c5fc18144fca1724713090c2ceeffa788c02711|type:100|phones:1234567,2345678,age:1|rust:5,3";

        assert_eq!(serialize_signature(msg).unwrap(), result)
    }

    #[test]
    fn signature_serialize_works_with_raw_hash_for_not_attrib_related_type() {
        let data = r#"{
                        "name": "John Doe",
                        "age": 43,
                        "operation": {
                            "type": "101",
                            "hash": "cool hash",
                            "dest": 54,
                            "raw": "string for hash"
                        },
                        "phones": [
                          "1234567",
                          "2345678",
                          {"rust": 5, "age": 1},
                          3
                        ]
                    }"#;
        let msg: Value = serde_json::from_str(data).unwrap();

        let result = "age:43|name:John Doe|operation:dest:54|hash:cool hash|raw:string for hash|type:101|phones:1234567,2345678,age:1|rust:5,3";

        assert_eq!(serialize_signature(msg).unwrap(), result)
    }


    #[test]
    fn signature_serialize_works_with_null() {
        let data = r#"{"signature": null}"#;
        let v: serde_json::Value = serde_json::from_str(data).unwrap();
        let serialized = serialize_signature(v).unwrap();
        assert_eq!(serialized, "");
    }
}

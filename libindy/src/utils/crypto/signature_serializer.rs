pub use indy_vdr::utils::signature::serialize_signature;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

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

        assert_eq!(serialize_signature(&msg).unwrap(), result)
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

        assert_eq!(serialize_signature(&msg).unwrap(), result)
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

        assert_eq!(serialize_signature(&msg).unwrap(), result)
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

        assert_eq!(serialize_signature(&msg).unwrap(), result)
    }


    #[test]
    fn signature_serialize_works_with_null() {
        let data = r#"{"signature": null}"#;
        let v: serde_json::Value = serde_json::from_str(data).unwrap();
        let serialized = serialize_signature(&v).unwrap();
        assert_eq!(serialized, "");
    }
}

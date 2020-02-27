use regex::Regex;

lazy_static! {
    pub static ref REGEX: Regex = Regex::new("^[a-z0-9]+:([a-z0-9]+):(.*)$").unwrap();
}

pub fn qualify(entity: &str, prefix: &str, method: &str) -> String {
    format!("{}:{}:{}", prefix, method, entity)
}

pub fn to_unqualified(entity: &str) -> String {
    match REGEX.captures(entity) {
        None => entity.to_string(),
        Some(caps) => {
            caps.get(2).map(|m| m.as_str().to_string()).unwrap_or(entity.to_string())
        }
    }
}

pub fn method(entity: &str) -> Option<String> {
    match REGEX.captures(entity) {
        None => None,
        Some(caps) => {
            caps.get(1).map(|m| m.as_str().to_string())
        }
    }
}

pub fn is_fully_qualified(entity: &str) -> bool {
    REGEX.is_match(&entity)
}

#[macro_export]
macro_rules! qualifiable_type (($newtype:ident) => (

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct $newtype(pub String);

    impl $newtype {

        #[allow(dead_code)]
        pub fn get_method(&self) -> Option<String> {
            qualifier::method(&self.0)
        }
        #[allow(dead_code)]
        pub fn set_method(&self, method: &str) -> $newtype {
            $newtype(qualifier::qualify(&self.0, $newtype::PREFIX, &method))
        }

        #[allow(dead_code)]
        pub fn is_fully_qualified(&self) -> bool {
            self.0.starts_with($newtype::PREFIX) && qualifier::is_fully_qualified(&self.0)
        }
    }
));
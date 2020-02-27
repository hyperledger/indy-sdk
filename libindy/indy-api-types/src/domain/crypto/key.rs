
use zeroize::Zeroize;

use named_type::NamedType;


#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Serialize, Deserialize, Clone, NamedType, PartialEq)]
pub struct Key {
    pub verkey: String,
    #[cfg(not(test))]
    #[derivative(Debug = "ignore")]
    pub signkey: String,
    #[cfg(test)]
    pub signkey: String,
}

impl Key {
    pub fn new(verkey: String, signkey: String) -> Key {
        Key {
            verkey,
            signkey,
        }
    }
}

impl Zeroize for Key {
    fn zeroize(&mut self) {
        self.signkey.zeroize();
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.signkey.zeroize();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyInfo {
    pub seed: Option<String>,
    pub crypto_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, NamedType)]
pub struct KeyMetadata {
    pub value: String
}

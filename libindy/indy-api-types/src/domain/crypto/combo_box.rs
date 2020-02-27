use rmp_serde;

#[derive(Serialize, Deserialize, Debug)]
pub struct ComboBox {
    pub msg: String,
    pub sender: String,
    pub nonce: String
}

impl ComboBox {
    pub fn to_msg_pack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::encode::to_vec_named(self)
    }

    pub fn from_msg_pack(bytes: &[u8]) -> Result<ComboBox, rmp_serde::decode::Error> {
        rmp_serde::decode::from_slice(bytes)
    }
}
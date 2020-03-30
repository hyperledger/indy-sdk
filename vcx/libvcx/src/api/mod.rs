pub mod vcx;
pub mod connection;
pub mod issuer_credential;
pub mod utils;
pub mod proof;
pub mod credential_def;
pub mod schema;
pub mod credential;
pub mod disclosed_proof;
pub mod wallet;
pub mod logger;
pub mod return_types_u32;

use std::fmt;

/// This macro allows the VcxStateType to be
/// serialized within serde as an integer (represented as
/// a string, because its still JSON).
macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                        where E: ::serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

enum_number!(VcxStateType
{
    VcxStateNone = 0,
    VcxStateInitialized = 1,
    VcxStateOfferSent = 2,
    VcxStateRequestReceived = 3,
    VcxStateAccepted = 4,
    VcxStateUnfulfilled = 5,
    VcxStateExpired = 6,
    VcxStateRevoked = 7,
    VcxStateRedirected = 8,
    VcxStateRejected = 9,
});

impl VcxStateType {
    pub fn from_u32(state: u32) -> VcxStateType {
        match state {
            0 => VcxStateType::VcxStateNone,
            1 => VcxStateType::VcxStateInitialized,
            2 => VcxStateType::VcxStateOfferSent,
            3 => VcxStateType::VcxStateRequestReceived,
            4 => VcxStateType::VcxStateAccepted,
            5 => VcxStateType::VcxStateUnfulfilled,
            6 => VcxStateType::VcxStateExpired,
            7 => VcxStateType::VcxStateRevoked,
            _ => VcxStateType::VcxStateNone,
        }
    }
}

// undefined is correlated with VcxStateNon -> Haven't received Proof
// Validated is both validated by indy-sdk and by comparing proof-request
// Invalid is that it failed one or both of validation processes
enum_number!(ProofStateType
{
    ProofUndefined = 0,
    ProofValidated = 1,
    ProofInvalid = 2,
});

enum_number!(PublicEntityStateType
{
    Built = 0,
    Published = 1,
});

impl Default for PublicEntityStateType{
    fn default() -> Self {
        PublicEntityStateType::Published
    }
}

impl Default for VcxStateType{
    fn default() -> Self {
        VcxStateType::VcxStateNone
    }
}

#[repr(C)]
pub struct VcxStatus {
    pub handle: libc::c_int,
    pub status: libc::c_int,
    pub msg: *mut libc::c_char,
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use self::VcxStateType::*;

    #[test]
    fn test_serialize_vcx_state_type(){
        let z = VcxStateNone;
        let y = serde_json::to_string(&z).unwrap();
        assert_eq!(y,"0");
    }
}

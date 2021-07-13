#[macro_use]
pub mod sodium_type;

#[cfg(feature = "chacha20poly1305_ietf_sodium")]
#[path = "chacha20poly1305_ietf/sodium.rs"]
pub mod chacha20poly1305_ietf;

#[cfg(feature = "hash_openssl")]
#[path = "hash/openssl.rs"]
pub mod hash;

#[cfg(feature = "hmacsha256_sodium")]
#[path = "hmacsha256/sodium.rs"]
pub mod hmacsha256;

#[cfg(feature = "pwhash_argon2i13_sodium")]
#[path = "pwhash_argon2i13/sodium.rs"]
pub mod pwhash_argon2i13;

#[cfg(feature = "randombytes_sodium")]
#[path = "randombytes/sodium.rs"]
pub mod randombytes;

#[cfg(feature = "sealedbox_sodium")]
#[path = "sealedbox/sodium.rs"]
pub mod sealedbox;

#[allow(dead_code)] /* FIXME Do we really need this module? */
#[cfg(feature = "xsalsa20_sodium")]
#[path = "xsalsa20/sodium.rs"]
pub mod xsalsa20;

#[cfg(feature = "ed25519_sign_sodium")]
#[path = "ed25519_sign/sodium.rs"]
pub mod ed25519_sign;

#[cfg(feature = "ed25519_box_sodium")]
#[path = "ed25519_box/sodium.rs"]
// TODO: The name is misleading as the operations do not happen over ed25519 curve
pub mod ed25519_box;

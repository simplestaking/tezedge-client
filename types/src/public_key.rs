use std::convert::TryInto;
use std::fmt::{self, Debug};
use serde::{Serialize, Serializer, Deserialize, Deserializer};

use crypto::{blake2b, Prefix, WithPrefix, WithoutPrefix, NotMatchingPrefixError};
use crypto::base58check::{FromBase58Check, ToBase58Check};
use crate::ImplicitAddress;
use super::FromPrefixedBase58CheckError;

pub const PUBLIC_KEY_LEN: usize = 32;
type PublicKeyInner = [u8; PUBLIC_KEY_LEN];

#[allow(non_camel_case_types)]
#[derive(PartialEq, Clone)]
pub enum PublicKey {
    edpk([u8; 32]),
    sppk([u8; 33]),
    p2pk([u8; 33]),
}

impl PublicKey {
    fn try_without_prefix(
        bytes: &[u8],
        prefix: Prefix,
    ) -> Result<(Prefix, Vec<u8>), NotMatchingPrefixError>
    {
        bytes
            .without_prefix(prefix)
            .map(|bytes| (prefix, bytes))
    }

    fn try_into_inner<T, U>(bytes: U) -> Result<T, FromPrefixedBase58CheckError>
        where U: TryInto<T>,
    {
        bytes.try_into().or(Err(FromPrefixedBase58CheckError::InvalidSize))
    }

    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        let bytes = encoded.from_base58check()?;

        let (prefix, bytes_vec) = Self::try_without_prefix(&bytes, Prefix::edpk)
            .or_else(|_| Self::try_without_prefix(&bytes, Prefix::sppk))
            .or_else(|_| Self::try_without_prefix(&bytes, Prefix::p2pk))?;

        match prefix {
            Prefix::edpk => Ok(Self::edpk(Self::try_into_inner(bytes_vec)?)),
            Prefix::sppk => Ok(Self::sppk(Self::try_into_inner(bytes_vec)?)),
            Prefix::p2pk => Ok(Self::p2pk(Self::try_into_inner(bytes_vec)?)),
            _ => unreachable!(),
        }
    }

    fn curve_prefix(&self) -> u8 {
        match self {
            Self::edpk(_) => 0x01,
            Self::sppk(_) => 0x02,
            Self::p2pk(_) => 0x03,
        }
    }

    /// Hash of the public key, gives it's corresponding address.
    pub fn hash(&self) -> ImplicitAddress {
        let key = match self {
            Self::edpk(key) => &key[..],
            Self::sppk(key) => &key[..],
            Self::p2pk(key) => &key[..],
        };
        let hash = blake2b::digest_160(key);
        let addr_bytes: [u8; 20] = hash
            .try_into()
            // unwrap can't fail since blake2b outputs 160 bit digest,
            // which is exactly 20 bytes, same as `addr_bytes` size.
            .unwrap();

        match self {
            Self::edpk(_) => ImplicitAddress::tz1(addr_bytes),
            Self::sppk(_) => ImplicitAddress::tz2(addr_bytes),
            Self::p2pk(_) => ImplicitAddress::tz3(addr_bytes),
        }
    }
}

impl ToBase58Check for PublicKey {
    fn to_base58check(&self) -> String {
        self.as_ref()
            .with_prefix(Prefix::edpk)
            .to_base58check()
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::edpk(k) => k,
            Self::sppk(k) => k,
            Self::p2pk(k) => k,
        }
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey(\"{}\")", self.to_base58check())
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(
            &self.to_base58check()
        )
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;

        Self::from_base58check(&encoded)
            .map_err(|err| {
                serde::de::Error::custom(err)
            })
    }
}

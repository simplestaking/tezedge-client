use std::convert::TryInto;
use std::fmt::{self, Debug};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crypto::{Prefix, WithPrefix, WithoutPrefix, NotMatchingPrefixError};
use crypto::base58check::{FromBase58Check, ToBase58Check};
use crate::FromPrefixedBase58CheckError;
use super::ADDRESS_LEN;

type ImplicitAddressInner = [u8; ADDRESS_LEN];

/// Implicit public key hash of the account.
///
/// Implicit are normal accounts (NOT smart contracts) prefixed with (tz1, tz2, tz3).
#[allow(non_camel_case_types)]
#[derive(PartialEq, Clone)]
pub enum ImplicitAddress {
    tz1(ImplicitAddressInner),
    tz2(ImplicitAddressInner),
    tz3(ImplicitAddressInner),
}

impl ImplicitAddress {
    fn try_without_prefix(
        bytes: &[u8],
        prefix: Prefix,
    ) -> Result<(Prefix, Vec<u8>), NotMatchingPrefixError>
    {
        bytes
            .without_prefix(prefix)
            .map(|bytes| (prefix, bytes))
    }

    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        let bytes = encoded.from_base58check()?;

        let (prefix, bytes_vec) = Self::try_without_prefix(&bytes, Prefix::tz1)
            .or_else(|_| Self::try_without_prefix(&bytes, Prefix::tz2))
            .or_else(|_| Self::try_without_prefix(&bytes, Prefix::tz3))?;

        let inner = bytes_vec.try_into()
            .or(Err(FromPrefixedBase58CheckError::InvalidSize))?;

        match prefix {
            Prefix::tz1 => Ok(Self::tz1(inner)),
            Prefix::tz2 => Ok(Self::tz2(inner)),
            Prefix::tz3 => Ok(Self::tz3(inner)),
            _ => unreachable!(),
        }
    }

    pub fn get_prefix(&self) -> Prefix {
        match self {
            Self::tz1(_) => Prefix::tz1,
            Self::tz2(_) => Prefix::tz2,
            Self::tz3(_) => Prefix::tz3,
        }
    }
}

impl ToBase58Check for ImplicitAddress {
    fn to_base58check(&self) -> String {
        self.as_ref().with_prefix(self.get_prefix()).to_base58check()
    }
}

impl AsRef<ImplicitAddressInner> for ImplicitAddress {
    fn as_ref(&self) -> &ImplicitAddressInner {
        match self {
            Self::tz1(k) => &k,
            Self::tz2(k) => &k,
            Self::tz3(k) => &k,
        }
    }
}

impl Debug for ImplicitAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let addr = self.to_base58check();
        match self {
            Self::tz1(_) => write!(f, "ImplicitAddress::tz1(\"{}\")", addr),
            Self::tz2(_) => write!(f, "ImplicitAddress::tz2(\"{}\")", addr),
            Self::tz3(_) => write!(f, "ImplicitAddress::tz3(\"{}\")", addr),
        }
    }
}

impl Serialize for ImplicitAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(
            &self.to_base58check()
        )
    }
}

impl<'de> Deserialize<'de> for ImplicitAddress {
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

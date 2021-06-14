use std::convert::TryInto;
use std::fmt::{self, Debug};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crypto::{Prefix, WithPrefix, WithoutPrefix};
use crypto::base58check::{FromBase58Check, ToBase58Check};
use crate::{ImplicitAddress, FromPrefixedBase58CheckError};
use super::ADDRESS_LEN;

type OriginatedAddressInner = [u8; ADDRESS_LEN];

/// Originated public key hash of the account.
///
/// Originated are essentially smart contracts, which are prefixed with (KT1).
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct OriginatedAddress(OriginatedAddressInner);

impl OriginatedAddress {
    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        let inner = encoded
            .from_base58check()?
            .without_prefix(Prefix::KT1)?
            .try_into()
            .or(Err(FromPrefixedBase58CheckError::InvalidSize))?;

        Ok(Self(inner))
    }

    pub fn get_prefix(&self) -> Prefix {
        Prefix::KT1
    }

    pub fn with_manager(self, manager_addr: ImplicitAddress) -> OriginatedAddressWithManager {
        OriginatedAddressWithManager {
            address: self,
            manager: manager_addr,
        }
    }
}

impl ToBase58Check for OriginatedAddress {
    fn to_base58check(&self) -> String {
        self.as_ref().with_prefix(self.get_prefix()).to_base58check()
    }
}

impl AsRef<OriginatedAddressInner> for OriginatedAddress {
    fn as_ref(&self) -> &OriginatedAddressInner {
        &self.0
    }
}

impl Debug for OriginatedAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OriginatedAddress(\"{}\")", self.to_base58check())
    }
}

impl Serialize for OriginatedAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(
            &self.to_base58check()
        )
    }
}

impl<'de> Deserialize<'de> for OriginatedAddress {
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

impl From<OriginatedAddressWithManager> for OriginatedAddress {
    fn from(s: OriginatedAddressWithManager) -> Self {
        s.address
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct OriginatedAddressWithManager {
    pub address: OriginatedAddress,
    pub manager: ImplicitAddress,
}

impl AsRef<OriginatedAddress> for OriginatedAddressWithManager {
    fn as_ref(&self) -> &OriginatedAddress {
        &self.address
    }
}

impl AsRef<[u8]> for OriginatedAddressWithManager {
    fn as_ref(&self) -> &[u8] {
        self.address.as_ref()
    }
}

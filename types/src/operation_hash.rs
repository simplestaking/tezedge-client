use std::convert::TryInto;
use std::fmt::{self, Debug};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crypto::{Prefix, WithPrefix, WithoutPrefix};
use crypto::base58check::{FromBase58Check, ToBase58Check};
use super::FromPrefixedBase58CheckError;

type OperationHashInner = [u8; 32];

#[derive(Eq, PartialEq, Clone)]
pub struct OperationHash(OperationHashInner);

impl OperationHash {
    /// Parse base58check.
    ///
    /// # Example
    /// ```rust
    /// # use types::OperationHash;
    /// OperationHash::from_base58check("onvQSsf5GZhrLPM98iHxBDk3Q1Xv3gQVGnicEpoyYMxNYxPdBhL").unwrap();
    /// ```
    pub fn from_base58check(encoded: &str) -> Result<Self, FromPrefixedBase58CheckError> {
        let key_bytes: OperationHashInner = encoded
            .from_base58check()?
            .without_prefix(Prefix::operation)?
            .try_into()
            .or(Err(FromPrefixedBase58CheckError::InvalidSize))?;

        Ok(Self(key_bytes))
    }
}

impl ToBase58Check for OperationHash {
    fn to_base58check(&self) -> String {
        self.0
            .with_prefix(Prefix::B)
            .to_base58check()
    }
}

impl AsRef<OperationHashInner> for OperationHash {
    fn as_ref(&self) -> &OperationHashInner {
        &self.0
    }
}

impl Debug for OperationHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OperationHash(\"{}\")", self.to_base58check())
    }
}

impl Serialize for OperationHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(
            &self.to_base58check()
        )
    }
}

impl<'de> Deserialize<'de> for OperationHash {
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

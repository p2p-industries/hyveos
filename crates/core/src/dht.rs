#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    grpc,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key {
    pub topic: String,
    pub key: Vec<u8>,
}

impl Key {
    /// Create a vector of bytes from the key.
    ///
    /// The key is a concatenation of the topic and the key, separated by a slash.
    ///
    /// # Errors
    ///
    /// Returns an error if the topic contains a slash.
    ///
    /// # Example
    ///
    /// ```
    /// use p2p_industries_core::dht::Key;
    ///
    /// let key = Key {
    ///    topic: "topic".to_string(),
    ///    key: "key".into(),
    /// };
    ///
    /// let bytes = key.into_bytes().unwrap();
    ///
    /// assert_eq!(bytes, b"topic/key");
    /// ```
    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let topic_bytes = self.topic.into_bytes();

        if topic_bytes.contains(&b'/') {
            return Err(Error::InvalidTopic);
        }

        Ok(topic_bytes
            .into_iter()
            .chain(Some(b'/'))
            .chain(self.key)
            .collect())
    }

    /// Create a key from a vector of bytes.
    ///
    /// The bytes should be a concatenation of the topic and the key, separated by a slash.
    ///
    /// # Errors
    ///
    /// Returns an error if the bytes do not contain a slash or if the topic is not valid UTF-8.
    ///
    /// # Example
    ///
    /// ```
    /// use p2p_industries_core::dht::Key;
    ///
    /// let bytes = b"topic/key".to_vec();
    /// let key = Key::from_bytes(bytes).unwrap();
    ///
    /// assert_eq!(key.topic, "topic");
    /// assert_eq!(&key.key, b"key");
    /// ```
    pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self> {
        let index = bytes
            .iter()
            .position(|b| *b == b'/')
            .ok_or(Error::InvalidKey("Should contain '/'".to_string()))?;

        let key = bytes.split_off(index + 1);
        bytes.pop();

        let topic = String::from_utf8(bytes).map_err(|e| Error::InvalidKey(e.to_string()))?;

        Ok(Self { topic, key })
    }
}

impl From<Key> for grpc::DhtKey {
    fn from(key: Key) -> Self {
        Self {
            topic: grpc::Topic { topic: key.topic },
            key: key.key,
        }
    }
}

impl From<grpc::DhtKey> for Key {
    fn from(key: grpc::DhtKey) -> Self {
        Self {
            topic: key.topic.topic,
            key: key.key,
        }
    }
}

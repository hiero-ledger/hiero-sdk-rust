use crate::proto::services;

use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::Key;

// note: it appears keylists "just" implement the APIs of arrays in their language, which means, uh...
// todo: Copy over the _entire_ `Vec` API?.
/// A list of keys with an optional threshold.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct KeyList {
    // todo: better doc comment?
    /// The list of keys.
    pub keys: Vec<Key>,

    /// If [`Some`]: The minimum number of keys that must sign.
    pub threshold: Option<u32>,
}

impl std::ops::Deref for KeyList {
    type Target = Vec<Key>;

    fn deref(&self) -> &Self::Target {
        &self.keys
    }
}

impl std::ops::DerefMut for KeyList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.keys
    }
}

impl KeyList {
    /// Create a new empty key list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if this keylist is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Removes and returns the element at position index within the key list, shifting all elements after it to the left.
    ///
    /// # Panics
    /// Panics if index is out of bounds.
    pub fn remove(&mut self, index: usize) -> Key {
        self.keys.remove(index)
    }

    // why not `ToProtobuf`? because `ToProtobuf` should return a `KeyList`.
    pub(crate) fn to_protobuf_key(&self) -> services::key::Key {
        let key_list = services::KeyList { keys: self.keys.to_protobuf() };

        if let Some(threshold) = self.threshold {
            return services::key::Key::ThresholdKey(services::ThresholdKey {
                threshold,
                keys: Some(key_list),
            });
        };

        services::key::Key::KeyList(key_list)
    }
}

impl ToProtobuf for KeyList {
    type Protobuf = services::KeyList;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::KeyList { keys: self.keys.to_protobuf() }
    }
}

impl FromIterator<Key> for KeyList {
    fn from_iter<T: IntoIterator<Item = Key>>(iter: T) -> Self {
        Self { keys: Vec::from_iter(iter), threshold: None }
    }
}

impl From<Vec<Key>> for KeyList {
    fn from(value: Vec<Key>) -> Self {
        Self { keys: value, threshold: None }
    }
}

impl<T: Into<Key>, const N: usize> From<[T; N]> for KeyList {
    fn from(value: [T; N]) -> Self {
        value.into_iter().map(Into::into).collect()
    }
}

impl FromProtobuf<services::KeyList> for KeyList {
    fn from_protobuf(pb: services::KeyList) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Vec::from_protobuf(pb.keys).map(Self::from)
    }
}

impl FromProtobuf<services::ThresholdKey> for KeyList {
    fn from_protobuf(pb: services::ThresholdKey) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let keys = Vec::from_protobuf(pb.keys.unwrap_or_default().keys)?;
        Ok(Self { keys, threshold: Some(pb.threshold) })
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use crate::proto::services;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::{
        KeyList,
        PrivateKey,
        PublicKey,
    };

    fn keys() -> [PublicKey; 3] {
        let key1 = PrivateKey::from_str_ed25519(
        "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e10").unwrap()
    .public_key();

        let key2 = PrivateKey::from_str_ed25519(
        "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e11").unwrap()
    .public_key();

        let key3 = PrivateKey::from_str_ed25519(
        "302e020100300506032b657004220420db484b828e64b2d8f12ce3c0a0e93a0b8cce7af1bb8f39c97732394482538e12").unwrap()
    .public_key();

        [key1, key2, key3]
    }

    #[test]
    fn from_protobuf() {
        let key_list_pb =
            services::KeyList { keys: keys().iter().map(|it| it.to_protobuf()).collect() };

        let key_list = KeyList::from_protobuf(key_list_pb).unwrap();

        assert!(keys().iter().all(|it| key_list.contains(&crate::Key::Single(*it))));
    }

    #[test]
    fn to_protobuf_key() {
        let key_list = KeyList::from(keys());

        let proto_key = key_list.to_protobuf_key();

        let proto_key_list = assert_matches!(proto_key, services::key::Key::KeyList(it) => it);

        for (actual, expected) in proto_key_list.keys.iter().zip(keys()) {
            assert_eq!(actual, &expected.to_protobuf());
        }
    }

    #[test]
    fn to_protobuf() {
        let key_list = KeyList::from(keys());

        let proto_key_list = key_list.to_protobuf();

        for (actual, expected) in proto_key_list.keys.iter().zip(keys()) {
            assert_eq!(actual, &expected.to_protobuf());
        }
    }

    #[test]
    fn len() {
        let key_list = KeyList::from(keys());
        let empty_key_list = KeyList::new();

        assert_eq!(key_list.len(), 3);
        assert!(!key_list.is_empty());
        assert_eq!(empty_key_list.len(), 0);
        assert!(empty_key_list.is_empty());
    }

    #[test]
    fn contains() {
        // Given / When

        let key_list = KeyList::from(keys());
        let empty_key_list = KeyList::new();

        assert!(keys().iter().all(|it| key_list.contains(&crate::Key::Single(*it))));
        assert!(!keys().iter().any(|it| empty_key_list.contains(&crate::Key::Single(*it))));
    }

    #[test]
    fn push() {
        let [a, b, c] = keys();

        let mut key_list = KeyList::from([a, b]);

        key_list.push(c.into());

        assert_eq!(key_list.len(), 3);

        assert!(key_list.contains(&c.into()));
    }

    #[test]
    fn remove() {
        let keys = keys();
        let mut key_list = KeyList::from(keys);

        let _ = key_list.remove(0);

        assert_eq!(key_list.len(), 2);

        assert!(!key_list.contains(&keys[0].into()));
        assert!(key_list.contains(&keys[1].into()));
        assert!(key_list.contains(&keys[2].into()));
    }

    #[test]
    fn clear() {
        let mut key_list = KeyList::from(keys());

        key_list.clear();

        assert!(key_list.is_empty());
    }
}

// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::collections::hash_map::{DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use parking_lot::RwLock;

use canonical::{Canon, CanonError, Sink, Source, Store};

#[derive(Default, Debug)]
struct MemStoreInner {
    map: HashMap<[u8; 8], Vec<u8>>,
    head: usize,
}

#[derive(Default, Debug, Clone)]
pub struct MemStore(Arc<RwLock<MemStoreInner>>);

impl MemStore {
    pub fn new() -> Self {
        Default::default()
    }
}

struct MemSink<S> {
    bytes: Vec<u8>,
    store: S,
}

struct MemSource<'a, S> {
    bytes: &'a [u8],
    offset: usize,
    store: S,
}

impl Store for MemStore {
    type Ident = [u8; 8];
    type Error = ();

    fn get<T: Canon<Self>>(&self, id: &Self::Ident) -> Result<T, CanonError> {
        self.0
            .read()
            .map
            .get(id)
            .map(|bytes| {
                let mut source = MemSource {
                    bytes,
                    offset: 0,
                    store: self.clone(),
                };
                T::read(&mut source)
            })
            .unwrap_or_else(|| Err(CanonError::MissingValue))
    }

    fn put_raw(&self, bytes: &[u8]) -> Result<Self::Ident, CanonError> {
        let mut hasher = DefaultHasher::new();
        bytes[..].hash(&mut hasher);
        let hash = hasher.finish().to_be_bytes();

        self.0.write().map.insert(hash, bytes.into());
        Ok(hash)
    }

    fn put<T: Canon<Self>>(&self, t: &T) -> Result<Self::Ident, CanonError> {
        let len = t.encoded_len();
        let mut bytes = Vec::with_capacity(len);
        unsafe {
            bytes.set_len(len);
        }
        Canon::write(t, &mut &mut bytes[..])?;
        self.put_raw(&mut bytes[..])
    }
}

impl<S: Store> Sink<S> for MemSink<S> {
    fn write_bytes(&mut self, n: usize) -> &mut [u8] {
        let ofs = self.bytes.len();
        self.bytes.resize_with(n, || 0);
        &mut self.bytes[ofs..]
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let ofs = self.bytes.len();
        self.bytes.resize_with(ofs + bytes.len(), || 0);
        self.bytes[ofs..].clone_from_slice(bytes)
    }

    fn recur(&mut self) -> Self {
        MemSink {
            bytes: vec![],
            store: self.store.clone(),
        }
    }

    fn fin(self) -> Result<S::Ident, CanonError> {
        self.store.put_raw(&self.bytes)
    }
}

impl<'a, S> Source<S> for MemSource<'a, S>
where
    S: Store,
{
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let ofs = self.offset;
        self.offset += n;
        &self.bytes[ofs..self.offset]
    }

    fn store(&self) -> S {
        self.store.clone()
    }
}

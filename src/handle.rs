use crate::{Canon, CanonError, Sink, Source, Store};
use core::marker::PhantomData;

/// The `Handle` type can be thought of as a host-allocating version of `Box`
pub struct Handle<T, S: Store> {
    Inline {
        bytes: S::Ident,
        len: u8,
        _marker: PhantomData<T>,
    },
    Ident(S::Ident),
}

impl<T, S> Canon<S> for Handle<T, S>
where
    S: Store,
{
    fn write(&mut self, sink: &mut impl Sink) {
        match self.inner {
            HandleInner::Inline {
                ref bytes,
                ref mut len,
                ..
            } => {
                Canon::<S>::write(&mut *len, sink);
                sink.copy_bytes(&bytes.as_ref()[0..*len as usize])
            }
            HandleInner::Ident(ref ident) => {
                Canon::<S>::write(&mut 0u8, sink);
                sink.copy_bytes(&ident.as_ref());
            }
        }
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError> {
        let len = u8::read(source)?;
        if len > 0 {
            let mut bytes = S::Ident::default();
            bytes.as_mut()[0..len as usize]
                .copy_from_slice(source.read_bytes(len as usize));
            Ok(Handle {
                store: source.store().clone(),
                inner: HandleInner::Inline {
                    bytes,
                    len,
                    _marker: PhantomData,
                },
            })
        } else {
            let mut ident = S::Ident::default();
            let bytes = source.read_bytes(ident.as_ref().len());
            ident.as_mut().copy_from_slice(bytes);
            Ok(Handle {
                store: source.store().clone(),
                inner: HandleInner::Ident(ident),
            })
        }
    }

    fn encoded_len(&self) -> usize {
        match &self.inner {
            HandleInner::Inline { len, .. } => {
                // length of tag + inline value
                1 + *len as usize
            }
            HandleInner::Ident(id) => 1 + id.as_ref().len(),
        }
    }
}

impl<T, S> Handle<T, S>
where
    T: Canon<S>,
    S: Store,
{
    /// Construct a new `Handle` from value `t`
    pub fn new(mut t: T) -> Result<Self, S::Error> {
        let mut buffer = S::Ident::default();
        let len = t.encoded_len();
        // can we inline the value?
        if len <= buffer.as_ref().len() {
            t.write(&mut buffer.as_mut());
            Ok(Handle {
                inner: HandleInner::Inline {
                    bytes: buffer,
                    len: len as u8,
                    _marker: PhantomData,
                },
            })
        } else {
            Ok(Handle {
                store: store.clone(),
                inner: HandleInner::Ident(store.put(&mut t)?),
            })
        }
    }

    /// Returns the value behind the `Handle`
    pub fn resolve(&self) -> Result<T, S::Error> {
        match &self.inner {
            HandleInner::Inline { bytes, len, .. } => {
                T::read(&mut &bytes.as_ref()[0..*len as usize])
                    .map_err(Into::into)
            }
            HandleInner::Ident(ident) => self.store.get(&ident),
        }
    }
}

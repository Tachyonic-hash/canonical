use crate::canon::{Canon, CanonError};

/// Restrictions on types acting as identifiers
pub trait Ident:
    Default + AsRef<[u8]> + AsMut<[u8]> + Clone + core::fmt::Debug
{
}
impl<T> Ident for T where
    T: Default + AsRef<[u8]> + AsMut<[u8]> + Clone + core::fmt::Debug
{
}

/// Trait to implement writing bytes to an underlying storage
pub trait Sink<S: Store> {
    /// Request n bytes to be written
    fn write_bytes(&mut self, n: usize) -> &mut [u8];
    /// Copy bytes from a slice into the `Sink`
    fn copy_bytes(&mut self, bytes: &[u8]);
    /// Recursively create another sink for storing children
    fn recur(&mut self) -> Self;
    /// Finish the sink, store the value, and return the identity
    fn fin(self) -> Result<S::Ident, CanonError<S::Error>>;
}

/// Trait to implement reading bytes from an underlying storage
pub trait Source<S> {
    /// Request n bytes from the sink to be read
    fn read_bytes(&mut self, n: usize) -> &[u8];
    /// Returns a copy of the Store associated with the source
    fn store(&self) -> S;
}

/// The main trait for storing/transmitting data, in the case of a wasm environment,
/// this is generally implemented with host calls
pub trait Store: Clone {
    /// The identifier used for allocations
    type Ident: Ident;
    /// The error the store can emit
    type Error: core::fmt::Debug;

    /// Get a value from storage, given an identifier
    fn get<T: Canon<Self>>(
        &self,
        id: &Self::Ident,
    ) -> Result<T, CanonError<Self::Error>>;

    /// Store raw bytes in the store
    fn put(&self, bytes: &[u8])
        -> Result<Self::Ident, CanonError<Self::Error>>;

    #[cfg(feature = "bridge")]
    /// Only applicable to the bridge version
    fn singleton() -> Self;

    #[cfg(feature = "bridge")]
    /// Only applicable to the bridge version
    fn buffer() -> &'static mut [u8];
}

/// Hack to allow the derive macro to assume stores are `Canon`
#[doc(hidden)]
impl<S> Canon<S> for S
where
    S: Store,
{
    fn write(&self, _: &mut impl Sink<S>) -> Result<(), CanonError<S::Error>> {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }

    fn read(_: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }

    fn encoded_len(&self) -> usize {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }
}

impl<S: Store> Sink<S> for &mut [u8] {
    fn write_bytes(&mut self, n: usize) -> &mut [u8] {
        let slice = core::mem::replace(self, &mut []);
        let (a, b) = slice.split_at_mut(n);
        *self = b;
        a
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let n = bytes.len();
        let slice = core::mem::replace(self, &mut []);
        let (a, b) = slice.split_at_mut(n);
        *self = b;
        a.copy_from_slice(bytes)
    }

    fn recur(&mut self) -> Self {
        unimplemented!("Non-recursive sink")
    }

    fn fin(self) -> Result<S::Ident, CanonError<S::Error>> {
        unimplemented!("Non-recursive sink")
    }
}

impl<S> Source<S> for &[u8] {
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let slice = core::mem::replace(self, &[]);
        let (a, b) = slice.split_at(n);
        *self = b;
        a
    }

    fn store(&self) -> S {
        unimplemented!("Non-recursive source")
    }
}

#[derive(Clone, Debug)]
/// A store that does not store anything
pub struct VoidStore;

impl Store for VoidStore {
    type Ident = [u8; 0];
    type Error = ();

    fn get<T: Canon<Self>>(
        &self,
        _: &Self::Ident,
    ) -> Result<T, CanonError<Self::Error>> {
        Err(CanonError::MissingValue)
    }

    fn put(&self, _: &[u8]) -> Result<Self::Ident, CanonError<Self::Error>> {
        Ok([])
    }

    #[cfg(feature = "bridge")]
    fn singleton() -> Self {
        VoidStore
    }

    #[cfg(feature = "bridge")]
    fn buffer() -> &'static mut [u8] {
        &mut []
    }
}

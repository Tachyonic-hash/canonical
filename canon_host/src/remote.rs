// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use canonical::{Canon, Sink, Source, Store};

/// A representation of a Module of erased type, with its root state reachable
/// from the Id in the store.
#[derive(Debug, Clone)]
pub struct Remote<S: Store> {
    id: S::Ident,
    store: S,
}

impl<S: Store> Remote<S> {
    /// Create a new remote given the initial State and store reference
    pub fn new<T: Canon<S>>(from: T, store: S) -> Result<Self, S::Error> {
        let id = store.put(&from)?;
        Ok(Remote { id, store })
    }

    /// Attempt casting this Remote to type `T`
    pub fn cast<T: Canon<S>>(&self) -> Result<T, S::Error> {
        self.store.get(&self.id)
    }

    /// Attempt casting this Remote to a mutable reference to type `T`
    pub fn cast_mut<T: Canon<S>>(&mut self) -> Result<CastMut<T, S>, S::Error> {
        let t = self.store.get(&self.id)?;
        Ok(CastMut {
            remote: self,
            value: t,
        })
    }
}

/// A cast of a remote to type `T`
#[derive(Debug)]
pub struct CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    /// The remote this CastMut derived from
    remote: &'a mut Remote<S>,
    /// The parsed value
    value: T,
}

impl<'a, T, S> Deref for CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<'a, T, S> DerefMut for CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<'a, T, S> CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    /// Commits the possibly changed value to the remote it was cast from
    pub fn commit(&mut self) -> Result<(), S::Error> {
        let id = self.remote.store.put(&self.value)?;
        self.remote.id = id;
        Ok(())
    }
}

impl<S: Store> Canon<S> for Remote<S> {
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        sink.copy_bytes(self.id.as_ref());
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let mut id = S::Ident::default();
        let slice = id.as_mut();
        let len = slice.len();
        slice.copy_from_slice(source.read_bytes(len));
        Ok(Remote {
            id,
            store: source.store().clone(),
        })
    }

    fn encoded_len(&self) -> usize {
        S::Ident::default().as_ref().len()
    }
}

/// Represents the type of a query
#[derive(Debug)]
pub struct Query<A, R, const ID: u8> {
    /// Arguments, in form of a tuple or single value
    args: A,
    /// The expected return type
    _return: PhantomData<R>,
}

impl<A, R, const ID: u8> Query<A, R, ID> {
    /// Construct a new query with provided arguments
    pub fn new(args: A) -> Self {
        Query {
            args,
            _return: PhantomData,
        }
    }

    /// Returns a reference to the arguments of a query
    pub fn args(&self) -> &A {
        &self.args
    }
}

/// Represents the type of a transaction
#[derive(Debug)]
pub struct Transaction<A, R, const ID: u8> {
    /// Arguments, in form of a tuple or single value
    args: A,
    /// The expected return type
    _return: PhantomData<R>,
}

impl<A, R, const N: u8> Transaction<A, R, N> {
    /// Create a new transaction
    pub fn new(args: A) -> Self {
        Transaction {
            args,
            _return: PhantomData,
        }
    }

    /// Returns a reference to the transactions arguments
    pub fn args(&self) -> &A {
        &self.args
    }

    /// Consumes transaction and returns the argument
    pub fn into_args(self) -> A {
        self.args
    }
}

/// Trait to support applying transactions
pub trait Apply<A, R, S, const ID: u8>
where
    S: Store,
{
    /// Apply a transaction to `self`
    fn apply(
        &mut self,
        transaction: Transaction<A, R, ID>,
    ) -> Result<R, S::Error>;
}

/// Trait to support executing queries
pub trait Execute<A, R, S, const ID: u8>
where
    S: Store,
{
    /// Execute a query over `self`
    fn execute(&self, query: Query<A, R, ID>) -> Result<R, S::Error>;
}

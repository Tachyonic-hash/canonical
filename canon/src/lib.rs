// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

//! Canonical, a no_std, host-allocating serialization library
#![cfg_attr(feature = "hosted", no_std)]
#![deny(missing_docs)]

mod canon;

#[cfg(feature = "hosted")]
mod bridge;
#[cfg(feature = "hosted")]
mod repr_hosted;
#[cfg(feature = "hosted")]
pub use bridge::BridgeStore;
#[cfg(feature = "hosted")]
pub use repr_hosted::Repr;

#[cfg(feature = "host")]
mod repr_host;
#[cfg(feature = "host")]
pub use repr_host::Repr;

mod dry_sink;
mod id;
mod implementations;
mod store;

pub use canon::{Canon, InvalidEncoding};
pub use dry_sink::DrySink;
pub use id::Id32;
pub use store::{ByteSink, ByteSource, IdBuilder, Ident, Sink, Source, Store};

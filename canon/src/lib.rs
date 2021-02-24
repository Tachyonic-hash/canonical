// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Canonical, a no_std, host-allocating serialization library
#![cfg_attr(not(feature = "host"), no_std)]
#![feature(never_type)]
#![deny(missing_docs)]

extern crate alloc;

mod canon;
mod debug;
mod id;
mod implementations;
mod repr;
mod store;

mod bridge;

pub use canon::{Canon, CanonError};
pub use debug::{DebugMsg, _debug};
pub use id::{Id, IdBuilder};
pub use repr::{Repr, Val, ValMut};
pub use store::{Sink, Source, Store};

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(lang_items)]

use canonical::Canon;
use canonical_derive::Canon;

// query ids
pub const READ_VALUE: u8 = 0;
pub const XOR_VALUE: u8 = 1;
pub const IS_EVEN: u8 = 2;

// transaction ids
pub const INCREMENT: u8 = 0;

#[derive(Clone, Canon, Debug)]
pub struct Counter {
    junk: u32,
    value: i32,
}

impl Counter {
    pub fn new(value: i32) -> Self {
        Counter {
            value,
            junk: 0xffffffff,
        }
    }
}

#[cfg(not(feature = "host"))]
mod hosted {
    use super::*;

    use canonical::{BridgeStore, ByteSink, ByteSource, Id32, Store};

    const PAGE_SIZE: usize = 1024 * 4;

    type BS = BridgeStore<Id32>;

    impl Counter {
        pub fn read_value(&self) -> i32 {
            self.value
        }

        pub fn xor_values(&self, a: i32, b: i32) -> i32 {
            self.value ^ a ^ b
        }

        pub fn is_even(&self) -> bool {
            self.value % 2 == 0
        }

        pub fn increment(&mut self) {
            self.value += 1;
        }

        pub fn decrement(&mut self) {
            self.value -= 1;
        }

        pub fn adjust(&mut self, by: i32) {
            self.value += by;
        }

        pub fn compare_and_swap(&mut self, expected: i32, new: i32) -> bool {
            if self.value == expected {
                self.value = new;
                true
            } else {
                false
            }
        }
    }

    fn query(bytes: &mut [u8; PAGE_SIZE]) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(&bytes[..], store.clone());

        // read self.
        let slf: Counter = Canon::<BS>::read(&mut source)?;

        // read query id
        let qid: u8 = Canon::<BS>::read(&mut source)?;
        match qid {
            // read_value (&Self) -> i32
            READ_VALUE => {
                let ret = slf.read_value();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                Canon::<BS>::write(&ret, &mut sink)?;
                Ok(())
            }
            // xor_values (&Self, a: i32, b: i32) -> i32
            XOR_VALUE => {
                let (a, b): (i32, i32) = Canon::<BS>::read(&mut source)?;
                let ret = slf.xor_values(a, b);
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                Canon::<BS>::write(&ret, &mut sink)?;
                Ok(())
            }
            // is_even (&Self) -> bool
            IS_EVEN => {
                let ret = slf.is_even();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());

                Canon::<BS>::write(&ret, &mut sink)?;
                Ok(())
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn q(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        let _ = query(bytes);
    }

    fn transaction(
        bytes: &mut [u8; PAGE_SIZE],
    ) -> Result<(), <BS as Store>::Error> {
        let store = BS::default();
        let mut source = ByteSource::new(bytes, store.clone());

        // read self.
        let mut slf: Counter = Canon::<BS>::read(&mut source)?;
        // read transaction id
        let qid: u16 = Canon::<BS>::read(&mut source)?;
        match qid {
            // increment (&Self)
            0 => {
                slf.increment();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // no return value
                Ok(())
            }
            1 => {
                // no args
                slf.decrement();
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // no return value
                Ok(())
            }
            2 => {
                // read arg
                let by: i32 = Canon::<BS>::read(&mut source)?;
                slf.adjust(by);
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // no return value
                Ok(())
            }
            3 => {
                // read multiple args
                let (a, b): (i32, i32) = Canon::<BS>::read(&mut source)?;
                let res = slf.compare_and_swap(a, b);
                let mut sink = ByteSink::new(&mut bytes[..], store.clone());
                // return new state
                Canon::<BS>::write(&slf, &mut sink)?;
                // return result
                Canon::<BS>::write(&res, &mut sink)
            }
            _ => panic!(""),
        }
    }

    #[no_mangle]
    fn t(bytes: &mut [u8; PAGE_SIZE]) {
        // todo, handle errors here
        transaction(bytes).unwrap()
    }

    mod panic_handling {
        pub fn signal(msg: &str) {
            let bytes = msg.as_bytes();
            let len = bytes.len() as u32;
            unsafe { sig(&bytes[0], len) }
        }

        #[link(wasm_import_module = "canon")]
        extern "C" {
            fn sig(msg: &u8, len: u32);
        }

        use core::fmt::{self, Write};
        use core::panic::PanicInfo;

        impl Write for PanicMsg {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                let bytes = s.as_bytes();
                let len = bytes.len();
                self.buf[self.ofs..self.ofs + len].copy_from_slice(bytes);
                self.ofs += len;
                Ok(())
            }
        }

        struct PanicMsg {
            ofs: usize,
            buf: [u8; 1024],
        }

        impl AsRef<str> for PanicMsg {
            fn as_ref(&self) -> &str {
                core::str::from_utf8(&self.buf[0..self.ofs])
                    .unwrap_or("PanicMsg.as_ref failed.")
            }
        }

        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            let mut msg = PanicMsg {
                ofs: 0,
                buf: [0u8; 1024],
            };

            writeln!(msg, "{}", info).ok();

            signal(msg.as_ref());

            loop {}
        }

        #[lang = "eh_personality"]
        extern "C" fn eh_personality() {}
    }
}

#[cfg(feature = "host")]
mod host {
    use super::*;
    use canonical_host::{Query, Transaction};

    impl Counter {
        pub fn read_value() -> Query<Self, (), i32, READ_VALUE> {
            Query::new(())
        }

        pub fn xor_values(
            a: i32,
            b: i32,
        ) -> Query<Self, (i32, i32), i32, XOR_VALUE> {
            Query::new((a, b))
        }

        pub fn is_even() -> Query<Self, (), bool, IS_EVEN> {
            Query::new(())
        }
    }

    impl Counter {
        pub fn increment() -> Transaction<Self, (), (), INCREMENT> {
            Transaction::new(())
        }

        pub fn decrement() -> Transaction<Self, (), (), 1> {
            Transaction::new(())
        }

        pub fn adjust(by: i32) -> Transaction<Self, i32, (), 2> {
            Transaction::new(by)
        }

        pub fn compare_and_swap(
            current: i32,
            new: i32,
        ) -> Transaction<Self, (i32, i32), bool, 3> {
            Transaction::new((current, new))
        }
    }
}

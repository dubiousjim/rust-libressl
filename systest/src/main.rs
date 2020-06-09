#![allow(bad_style, clippy::all)]

extern crate libc;
extern crate libressl_sys;

use libc::*;
use libressl_sys::*;

include!(concat!(env!("OUT_DIR"), "/all.rs"));

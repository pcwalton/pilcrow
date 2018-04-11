// pilcrow/build.rs
//
// Copyright © 2018 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate cbindgen;

use cbindgen::{Builder, Config};
use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let config = Config::from_file("cbindgen.toml").expect("Failed to read `cbindgen.toml`!");
    Builder::new().with_crate(crate_dir)
                  .with_config(config)
                  .generate()
                  .expect("Unable to generate bindings!")
                  .write_to_file("pilcrow.h");
}

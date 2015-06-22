Strscan
=======

Simple rust library for matching text against regular expressions for use in
lexers or other software.

Basic Usage
-----------
```rust

extern crate strscan;
use strscan::StringScanner;

let s = StringScanner::new(input_string);
let r_chars = Regex::new(r"\A\s*(\w+)\s*")

let res = s.scan(r_chars)
println!("chars with possible space: {}", res.unwrap());
println!("chars: {}", s.match_at(1).unwrap());
```

Rust Compatibility
------------------

Uses the 'collections' feature in order to use unstable string APIs
(namely String#slice\_chars). Rust nightly only.

Latest tests with rustc 1.2.0-nightly (c6b148337 2015-06-12).

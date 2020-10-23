# union_type

[![license](https://img.shields.io/github/license/longfangsong/union_type.svg)](https://github.com/longfangsong/union_type/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/union_type.svg)](https://crates.io/crates/union_type)
[![Docs.rs](https://docs.rs/union_type/badge.svg)](https://docs.rs/union_type/)

Add union type support to rust!

## Why we need this?

See [here](./doc/why.md).

## What's the result look like?

```rust
#[macro_use]
extern crate union_type;

use std::convert::TryInto;
use std::fmt::Display;

#[derive(Debug, Clone)]
struct A(String);

impl A {
    fn f(&self, a: i32) -> i32 {
        println!("from A {}", a + 1);
        a + 1
    }

    fn g<T: Display>(&self, t: T) -> String {
        self.0.clone() + &format!("{}", t)
    }
}

#[derive(Debug, Clone)]
struct B(i32);

impl B {
    fn f(&self, a: i32) -> i32 {
        println!("from B {}", a + self.0);
        a + self.0
    }

    fn g<T: Display>(&self, t: T) -> String {
        format!("{}:{}", self.0, t)
    }
}

union_type! {
    #[derive(Debug, Clone)]
    enum C {
        A,
        B
    }
    impl C {
        fn f(&self, a: i32) -> i32;
        fn g<T: Display>(&self, t: T) -> String;
    }
}

fn main() {
    let a = A("abc".to_string());
    let mut c: C = a.into();
    c.f(1);
    let b = B(99);
    c = b.into();
    c.f(2);
    println!("{:?}", c);
    println!("{}", c.g(99));
    let b: B = c.try_into().unwrap();
    println!("{:?}", b);
}
```

The output is: 
```shell
from A 2
from B 101
B(B(99))
99:99
B(99)
```

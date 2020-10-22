#[macro_use]
extern crate union_type;

struct A(String);

impl A {
    fn f(&self, a: i32) -> i32 {
        println!("from A {}", a + 1);
        a + 1
    }
}

struct B(i32);

impl B {
    fn f(&self, a: i32) -> i32 {
        println!("from B {}", a + &self.0);
        a + &self.0
    }
}

union_type! {
    enum C {
        A,
        B
    }

    impl C {
        fn f(&self, a: i32) -> i32;
    }
}

fn main() {
    let a = A("abc".to_string());
    let c = C::A(a);
    c.f(1);
    let b = B(99);
    let c = C::B(b);
    c.f(2);
}
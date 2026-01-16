use crate::collections::vec::MyVec;

mod manager;
mod collections;
mod smart_pointers;


static S_INT: &'static u32 = &2;

fn main() {
    
}

#[derive(Debug)]
struct Foo {
    a: usize,
    b: i64,
    c: Deep,
}

#[derive(Debug)]
struct Deep {
    a: usize,
    b: u128
}

impl Foo {
    fn new(a: usize, b: i64) -> Foo {
        Foo { a, b, c: Deep { a, b: b as u128 } }
    }
}


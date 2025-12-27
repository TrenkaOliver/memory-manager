mod manager;
use manager ::Manager;

mod collections;
use crate::collections::MyVec;

fn main() {
    let mut heap = [0u8; 8192];
    let mut manager = Manager::new(&mut heap);

    let mut v = MyVec::new(&mut manager);

    v.push(Foo::new(0, 0));
    v.push(Foo::new(1, 1));
    v.push(Foo::new(2, 2));
    v.push(Foo::new(3, 3));
    v.push(Foo::new(4, 4));

    let mut other = MyVec::new(&mut manager);

    other.push(Foo::new(1, 123));
    other.push(Foo::new(123, 2));
    other.push(Foo::new(3, 123));
    other.insert(Foo::new(0, 0), 2);
    other.push(Foo::new(123, 4));
    other.push(Foo::new(5, 123));

    let removed = v.remove(2);
    let a = &mut v[2];
    a.c.a = 123141451;
    let popped = v.pop();


    v.append(other);

    for i in 0..v.len() {
        println!("{}: {:?}", i, v[i]);
    }

    println!("removed: {:?}", removed);
    println!("popped: {:?}", popped);

    manager.debug_free();
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
    b: i64
}

impl Foo {
    fn new(a: usize, b: i64) -> Foo {
        Foo { a, b, c: Deep { a, b } }
    }
}


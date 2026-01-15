mod manager;
mod collections;
mod smart_pointers;

use std::{mem, thread, time::Duration};

use collections::{MyVec, MyString};
use smart_pointers::{MyBox, MyRc, MyWeak};
use manager::{debug_free, my_alloc};

fn main() {
    {
        let mut handles = MyVec::new();
        for i in 1..=10 {
            let h = thread::spawn(move || {
                let mut v = MyVec::new();
                thread::sleep(Duration::from_millis(10 - i));
                for j in 1..=10 {
                    v.push(i * j);
                }
                println!("{:?}", v);
            });

            handles.push(h);
        }

        for h in handles {
            h.join().unwrap();
        }    
    }

    debug_free();
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


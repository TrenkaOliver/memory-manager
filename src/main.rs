mod manager;
mod collections;

use collections::{MyVec, MyString};
use manager::{debug_free, my_alloc};


fn main() {

    unsafe {
        my_alloc(1, 1);
    }

    let mut my_string = MyString::from_str("alma");
    my_string.push_str("\nkecske");
    my_string.push('\n');
    my_string.push('b');
    my_string.push('é');
    my_string.push('k');
    my_string.push('a');

    for line in my_string.lines() {
        println!("line: {}", line)
    }

    let mut my_string = MyString::from_str("🚀aá中a🚀中");
    my_string.insert(4, 'k');
    my_string.insert_str(11, "(inserted)");
    
    println!("my_string: {}", my_string);
    my_string.remove(8);
    println!("my_string: {}", my_string);

    drop(my_string);
    
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


use core::{str, fmt::Display, ops::{Add, AddAssign, Deref, DerefMut}};

use crate::collections::MyVec;

pub struct MyString {
    vec: MyVec<u8>
}

//constructors, getters
impl MyString {
    pub fn new() -> MyString {
        MyString { vec: MyVec::new() }
    }

    pub fn with_capacity(capacity: usize) -> MyString {
        MyString { vec: MyVec::with_capacity(capacity) }
    }

    pub fn from_str(s: &str) -> MyString {
        MyString { vec: MyVec::from_slice(s.as_bytes()) }
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.vec.as_slice())
        }
    }

    pub fn as_str_mut(&mut self) -> &mut str {
        unsafe {
            str::from_utf8_unchecked_mut(self.vec.as_slice_mut())
        }
    }
}

//adding elements
impl MyString {
    pub fn push(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let bytes = c.encode_utf8(&mut buf).as_bytes();
        self.vec.extend_from_slice(bytes);
    }

    pub fn push_str(&mut self, s: &str) {
        self.vec.extend_from_slice(s.as_bytes());
    }

    pub fn insert(&mut self, index: usize, c: char) {
        assert!(self.is_char_boundary(index));
        let mut buf = [0u8; 4];
        let bytes = c.encode_utf8(&mut buf).as_bytes();
        self.vec.insert_slice(index, bytes);
    }

    pub fn insert_str(&mut self, index: usize, s: &str) {
        assert!(self.is_char_boundary(index));
        self.vec.insert_slice(index, s.as_bytes());
    }
}

//removing elements
impl MyString {
    pub fn pop(&mut self) -> Option<char> {
        if let Some((idx, c)) = self.char_indices().next_back() {
            self.vec.drain(idx..);
            Some(c)
        } else {
            None
        }
    }

    pub fn remove(&mut self, index: usize) -> char {
        assert!(self.is_char_boundary(index));
       
        let s = unsafe {
            str::from_utf8_unchecked(&self.vec.as_slice()[index..])
        };
        let c = s.chars().next().unwrap();
        let end = c.len_utf8() + index;
        self.vec.drain(index..end);

        c
    }

    pub fn truncate(&mut self, len: usize) {
        assert!(self.is_char_boundary(len));
        self.vec.truncate(len);
    }

    
}

impl Add<&str> for MyString {
    type Output = Self;

    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl AddAssign<&str> for MyString {
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}

impl Deref for MyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl DerefMut for MyString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_str_mut()
    }
}

impl Display for MyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
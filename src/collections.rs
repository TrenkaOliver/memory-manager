use std::{ops::{Index, IndexMut}, ptr};

use crate::Manager;

pub struct MyVec<'a, T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
    manager: *mut Manager<'a>,
}

//constructors, getters
impl<'a, T> MyVec<'a, T> {
    pub fn new(manager: *mut Manager<'a>) -> MyVec<'a, T> {
        MyVec { ptr: ptr::null_mut(), len: 0, cap: 0, manager }
    }
    
    pub fn with_capacity(cap: usize, manager: *mut Manager<'a>) -> MyVec<'a, T> {
        let ptr = unsafe {
            (*manager).alloc(size_of::<T>() * cap) as *mut T
        };

        MyVec { ptr, len: 0, cap, manager }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

//adding values
impl<'a, T> MyVec<'a, T> {
    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.reallocate();
        }
        
        unsafe {
            *self.ptr.add(self.len) = value
        }

        self.len += 1;
    }

    pub fn insert(&mut self, value: T, index: usize) {
        if self.len == self.cap {
            self.reallocate();
        }

        unsafe {
            ptr::copy(self.ptr.add(index), self.ptr.add(index + 1), self.len - index);
            *self.ptr.add(index) = value;
        }

        self.len += 1;
    }

    pub fn append(&mut self, other: MyVec<'a, T>) {
        while self.len + other.len >= self.cap {
            self.reallocate();
        }

        unsafe {
            ptr::copy_nonoverlapping(other.ptr, self.ptr.add(self.len), other.len);
        }
        
        self.len += other.len;
    }

}

//removing values
impl<'a, T> MyVec<'a, T> {
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr.add(self.len)))
            }
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len);

        let removed = unsafe {
            ptr::read(self.ptr.add(index))
        };

        unsafe {
            ptr::copy(self.ptr.add(index + 1), self.ptr.add(index), self.len - index - 1);
        };

        self.len -= 1;

        removed
    }
}

//local helper functions
impl<'a, T> MyVec<'a, T> {
    fn reallocate(&mut self) {
        if self.cap == 0 {
            self.cap = 4;
            self.ptr = unsafe {
                (*self.manager).alloc(4 * size_of::<T>()) as *mut T
            };
            return;
        }

        let new_cap = {
            if self.cap <= 16 {
                self.cap * 2
            } else {
                self.cap + self.cap / 2
            }
        };

        let new_ptr = unsafe {
            (*self.manager).alloc(new_cap * size_of::<T>()) as *mut T
        };

        unsafe {
            ptr::copy_nonoverlapping(self.ptr, new_ptr, self.cap);
        }

        unsafe {
            (*self.manager).free(self.ptr);
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
    }
}

//access
impl<'a, T> Index<usize> for MyVec<'a, T>  {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);

        unsafe {
            & *self.ptr.add(index)
        }
    }
}

//mutable access
impl<'a, T> IndexMut<usize> for MyVec<'a, T>  {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len);

        unsafe {
            &mut *self.ptr.add(index)
        }
    }
}

//free memory when vec goes out of scope
impl<'a, T> Drop for MyVec<'a, T> {
    fn drop(&mut self) {
        unsafe {
            (*self.manager).free(self.ptr);
        }
    }
}
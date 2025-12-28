use std::{ops::{Index, IndexMut, RangeBounds}, ptr};

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
    
    pub fn with_capacity(capacity: usize, manager: *mut Manager<'a>) -> MyVec<'a, T> {
        let ptr = unsafe {
            (*manager).alloc(size_of::<T>() * capacity, align_of::<T>()) as *mut T
        };

        MyVec { ptr, len: 0, cap: capacity, manager }
    }

    pub fn from_slice(slice: &[T], manager: *mut Manager<'a>) -> MyVec<'a, T> {
        let mut v = MyVec::with_capacity(slice.len(), manager);
        v.extend_from_slice(slice);
        v
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }

    pub fn iter<'b>(&'b self) -> MyVecIter<'a, 'b, T> {
        MyVecIter { vec: self, index: 0 }
    }

    pub fn iter_mut<'b>(&'b mut self) -> MyVecIterMut<'a, 'b, T> {
        MyVecIterMut { vec: self, index: 0 }
    }
}

//adding values
impl<'a, T> MyVec<'a, T> {
    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.reallocate(None);
        }
        
        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }

        self.len += 1;
    }

    pub fn insert(&mut self, value: T, index: usize) {
        if self.len == self.cap {
            self.reallocate(None);
        }

        unsafe {
            ptr::copy(self.ptr.add(index), self.ptr.add(index + 1), self.len - index);
            ptr::write(self.ptr.add(index), value);
        }

        self.len += 1;
    }

    pub fn append(&mut self, other: MyVec<'a, T>) {
        let sum_len = self.len + other.len;
        if sum_len >= self.cap {
            self.reallocate(Some(sum_len));
        }

        unsafe {
            ptr::copy_nonoverlapping(other.ptr, self.ptr.add(self.len), other.len);
        }
        
        self.len += other.len;
    }

    pub fn extend_from_slice(&mut self, slice: &[T]) {
        let sum_len = self.len + slice.len();
        if sum_len >= self.cap {
            self.reallocate(Some(sum_len));
        }

        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), self.ptr.add(self.len), slice.len());
        }

        self.len += slice.len();
    }


}

//removing values
impl<'a, T> MyVec<'a, T> {
    pub fn clear(&mut self) {
        self.len = 0;
    }

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

    pub fn drain<'b, R>(&'b mut self, range: R) -> Drain<'a, 'b, T>
    where R: RangeBounds<usize> {
        let len = self.len;
        let start = match range.start_bound() {
            std::ops::Bound::Included(&s) => s,
            std::ops::Bound::Excluded(&s) => s + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&e) => e + 1,
            std::ops::Bound::Excluded(&e) => e,
            std::ops::Bound::Unbounded => len,
        };

        assert!(start <= end && end <= len);

        Drain { vec: self, index: start, start, end }
    }
}

//local helper functions
impl<'a, T> MyVec<'a, T> {
    fn reallocate(&mut self, to: Option<usize>) {
        if self.cap == 0 {
            self.cap = 4;
            self.ptr = unsafe {
                (*self.manager).alloc(4 * size_of::<T>(), align_of::<T>()) as *mut T
            };
            return;
        }

        let mut new_cap = {
            if self.cap <= 16 {
                self.cap * 2
            } else {
                self.cap + self.cap / 2
            }
        };

        if let Some(c) = to && c > new_cap {
            new_cap = c;
        }

        let new_ptr = unsafe {
            (*self.manager).alloc(new_cap * size_of::<T>(), align_of::<T>()) as *mut T
        };

        unsafe {
            println!("src: {}", self.ptr as usize);
            println!("dst: {}", new_ptr as usize);
            println!("len: {}", self.cap * size_of::<T>());

            println!("dst align: {}", new_ptr as usize % align_of::<T>());

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

//iterator implementations
impl<'a, T> IntoIterator for MyVec<'a, T> {
    type Item = T;

    type IntoIter = MyVecIntoIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let ptr = self.ptr;
        let index = 0;
        let len = self.len;
        let manager = self.manager;

        std::mem::forget(self);

        MyVecIntoIter {
            ptr,
            index,
            len,
            manager
        }
    }
}

impl<'a, 'b, T> IntoIterator for &'b MyVec<'a, T> {
    type Item = &'b T;

    type IntoIter = MyVecIter<'a, 'b, T>;

    fn into_iter(self) -> Self::IntoIter {
        MyVecIter {
            vec: self,
            index: 0
        }
    }
}

impl<'a, 'b, T> IntoIterator for &'b mut MyVec<'a, T> {
    type Item = &'b mut T;

    type IntoIter = MyVecIterMut<'a, 'b, T>;

    fn into_iter(self) -> Self::IntoIter {
        MyVecIterMut {
            vec: self,
            index: 0
        }
    }
}

//free memory when vec goes out of scope
impl<'a, T> Drop for MyVec<'a, T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len {
                drop(ptr::read(self.ptr.add(i)));
            }
            (*self.manager).free(self.ptr);
            println!("dropped");
        }
    }
}




//ITERATORS
pub struct MyVecIntoIter<'a, T> {
    ptr: *mut T,
    index: usize,
    len: usize,
    manager: *mut Manager<'a>,
}

impl<'a, T> Iterator for MyVecIntoIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let item = unsafe {
                let ptr = self.ptr.add(self.index);
                ptr::read(ptr)
            };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a, T> Drop for MyVecIntoIter<'a, T> {
    fn drop(&mut self) {
        unsafe {
            for i in self.index..self.len {
                ptr::drop_in_place(self.ptr.add(i));
            }
            (*self.manager).free(self.ptr);
        }
        println!("dropped");
    }
}

pub struct MyVecIter<'a, 'b, T> {
    vec: &'b MyVec<'a, T>,
    index: usize,
}

impl<'a, 'b, T> Iterator for MyVecIter<'a, 'b, T> {
    type Item = &'b T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let out = unsafe {
                & *self.vec.ptr.add(self.index)
            };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

pub struct MyVecIterMut<'a, 'b, T> {
    vec: &'b mut MyVec<'a, T>,
    index: usize,
}

impl<'a, 'b, T> Iterator for MyVecIterMut<'a, 'b, T> {
    type Item = &'b mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let out = unsafe {
                &mut *self.vec.ptr.add(self.index)
            };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.vec.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

pub struct Drain<'a, 'b, T> {
    vec: &'b mut MyVec<'a, T>,
    index: usize,
    start: usize,
    end: usize,
}

impl<'a, 'b, T> Iterator for Drain<'a, 'b, T>  {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.end {
            let out = unsafe {
                ptr::read(self.vec.ptr.add(self.index))
            };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a, 'b, T> Drop for Drain<'a, 'b, T> {
    fn drop(&mut self) {
        if self.end != self.vec.len { 
            unsafe {
                ptr::copy(self.vec.ptr.add(self.end), self.vec.ptr.add(self.start), self.vec.len - self.end);
            } 
        }
        self.vec.len -= self.end - self.start;
    }
}
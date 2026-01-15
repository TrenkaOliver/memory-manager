use core::{fmt::Debug, ops::{Index, IndexMut, RangeBounds}, ptr, marker::{Send, Sync}};

use crate::manager::{debug_free, my_alloc, my_free};

pub struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

//marker traits
unsafe impl<T: Send> Send for MyVec<T> {}
unsafe impl<T: Sync> Sync for MyVec<T> {}

//constructors, getters
impl<T> MyVec<T> {
    pub fn new() -> MyVec<T> {
        MyVec { ptr: ptr::null_mut(), len: 0, cap: 0 }
    }
    
    pub fn with_capacity(capacity: usize) -> MyVec<T> {
        let ptr = unsafe {
            my_alloc(size_of::<T>() * capacity, align_of::<T>()) as *mut T
        };

        MyVec { ptr, len: 0, cap: capacity }
    }

    pub fn from_slice(slice: &[T]) -> MyVec<T> {
        let mut v = MyVec::with_capacity(slice.len());
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

    pub fn as_slice_mut(&self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.len)
        }
    }

    pub fn iter<'a>(&'a self) -> MyVecIter<'a, T> {
        MyVecIter { vec: self, index: 0 }
    }

    pub fn iter_mut<'a>(&'a mut self) -> MyVecIterMut<'a, T> {
        MyVecIterMut { vec: self, index: 0 }
    }
}

//adding values
impl<T> MyVec<T> {
    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.reallocate(None);
        }
        
        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }

        self.len += 1;
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len);

        if self.len == self.cap {
            self.reallocate(None);
        }

        unsafe {
            ptr::copy(self.ptr.add(index), self.ptr.add(index + 1), self.len - index);
            ptr::write(self.ptr.add(index), value);
        }

        self.len += 1;
    }

    pub fn insert_slice(&mut self, index: usize, slice: &[T]) {
        assert!(index <= self.len);

        let sum_len = self.len + slice.len();
        if sum_len > self.cap {
            self.reallocate(Some(sum_len));
        }

        unsafe {
            ptr::copy(self.ptr.add(index), self.ptr.add(index + slice.len()), self.len - index);
            ptr::copy_nonoverlapping(slice.as_ptr(), self.ptr.add(index), slice.len());
        }

        self.len = sum_len;
    }

    pub fn append(&mut self, other: MyVec<T>) {
        let sum_len = self.len + other.len;
        if sum_len > self.cap {
            self.reallocate(Some(sum_len));
        }

        unsafe {
            ptr::copy_nonoverlapping(other.ptr, self.ptr.add(self.len), other.len);
        }
        
        self.len += other.len;
    }

    pub fn extend_from_slice(&mut self, slice: &[T]) {
        let sum_len = self.len + slice.len();
        if sum_len > self.cap {
            self.reallocate(Some(sum_len));
        }

        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), self.ptr.add(self.len), slice.len());
        }

        self.len = sum_len;
    }


}

//removing values
impl<T> MyVec<T> {
    pub fn clear(&mut self) {
        unsafe {
            for i in 0..self.len {
                drop(ptr::read(self.ptr.add(i)));
            }
        }
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

    pub fn truncate(&mut self, len: usize) {
        assert!(len <= self.len);
        unsafe {
            for i in len..self.len {
                drop(ptr::read(self.ptr.add(i)));
            }
        }
        self.len = len;
    }

    pub fn drain<'a, R>(&'a mut self, range: R) -> MyDrain<'a, T>
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

        let tail = self.len - end;
        self.len = start;

        MyDrain { vec: self, index: start, end, tail }
    }
}

//local helper functions
impl<T> MyVec<T> {
    fn reallocate(&mut self, to: Option<usize>) {
        if self.cap == 0 {
            self.cap = 4;
            self.ptr = unsafe {
                my_alloc(4 * size_of::<T>(), align_of::<T>()) as *mut T
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
            my_alloc(new_cap * size_of::<T>(), align_of::<T>()) as *mut T
        };

        unsafe {
            ptr::copy_nonoverlapping(self.ptr, new_ptr, self.cap);
            my_free(self.ptr);
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
    }
}

//access
impl<T> Index<usize> for MyVec<T>  {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);

        unsafe {
            & *self.ptr.add(index)
        }
    }
}


//mutable access
impl<T> IndexMut<usize> for MyVec<T>  {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len);

        unsafe {
            &mut *self.ptr.add(index)
        }
    }
}

//iterator implementations
impl<T> IntoIterator for MyVec<T> {
    type Item = T;

    type IntoIter = MyVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let ptr = self.ptr;
        let index = 0;
        let len = self.len;

        std::mem::forget(self);

        MyVecIntoIter {
            ptr,
            index,
            len,
        }
    }
}

impl<'a, T> IntoIterator for &'a MyVec<T> {
    type Item = &'a T;

    type IntoIter = MyVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        MyVecIter {
            vec: self,
            index: 0
        }
    }
}

impl<'a, T> IntoIterator for &'a mut MyVec<T> {
    type Item = &'a mut T;

    type IntoIter = MyVecIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        MyVecIterMut {
            vec: self,
            index: 0
        }
    }
}

//free memory when vec goes out of scope
impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len {
                ptr::drop_in_place(self.ptr.add(i));
            }
            my_free(self.ptr);
        }
    }
}

impl<T: Debug> Debug for MyVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //f.debug_struct("MyVec").field("ptr", &self.ptr).field("len", &self.len).field("cap", &self.cap).finish()
        let mut list = f.debug_list();
        for item in self {
            list.entry(item);
        };
        list.finish()
    }
}

//ITERATORS
pub struct MyVecIntoIter<T> {
    ptr: *mut T,
    index: usize,
    len: usize,
}

impl<T> Iterator for MyVecIntoIter<T> {
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

impl<T> Drop for MyVecIntoIter<T> {
    fn drop(&mut self) {
        unsafe {
            for i in self.index..self.len {
                ptr::drop_in_place(self.ptr.add(i));
            }
            my_free(self.ptr);
        }
    }
}

pub struct MyVecIter<'a, T> {
    vec: &'a MyVec<T>,
    index: usize,
}

impl<'a, T> Iterator for MyVecIter<'a, T> {
    type Item = &'a T;

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

pub struct MyVecIterMut<'a, T> {
    vec: &'a mut MyVec<T>,
    index: usize,
}

impl<'a, T> Iterator for MyVecIterMut<'a, T> {
    type Item = &'a mut T;

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

pub struct MyDrain<'a, T> {
    vec: &'a mut MyVec<T>,
    index: usize,
    end: usize,
    tail: usize,
}

impl<'a, T> Iterator for MyDrain<'a, T>  {
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

impl<'a, T> Drop for MyDrain<'a, T> {
    fn drop(&mut self) {
        for i in self.index..self.end {
            unsafe {
                ptr::drop_in_place(self.vec.ptr.add(i));
            }
        }

        if self.end != self.vec.len { 
            unsafe {
                ptr::copy(self.vec.ptr.add(self.end), self.vec.ptr.add(self.vec.len), self.tail);
            } 
        }
        
        self.vec.len += self.tail;
    }
}
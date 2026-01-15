use core::{fmt::Display, ops::{Deref, DerefMut}, ptr, marker::{Send, Sync}};

use crate::manager::{my_alloc, my_free};



pub struct MyBox<T> {
    ptr: *mut T,
}

unsafe impl<T: Send> Send for MyBox<T> {}
unsafe impl<T: Sync> Sync for MyBox<T> {}


impl<T> MyBox<T> {
    pub fn new(value: T) -> MyBox<T> {
        let ptr = unsafe {
            let ptr = my_alloc(size_of::<T>(), align_of::<T>()) as *mut T;
            ptr::write(ptr, value);
            ptr
        };
        

        MyBox { ptr }
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            & *self.ptr
        }
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.ptr
        }
    }
}

impl<T> Display for MyBox<T>
where T: Display  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &**self)
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.ptr);
            my_free(self.ptr);
        }
    }
}
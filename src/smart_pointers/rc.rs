use core::{fmt::Display, ops::{Deref, DerefMut}, ptr};

use crate::{manager::{my_alloc, my_free}, smart_pointers::MyBox};



pub struct MyRc<T> {
    strong_ptr: *mut usize,
    weak_ptr: *mut usize,
    value_ptr: *mut T,
}

impl<T> MyRc<T> {
    pub fn new(value: T) -> MyRc<T> {
        let (strong_ptr, weak_ptr, value_ptr) = unsafe {
            let ptr = my_alloc(size_of::<(usize, usize, T)>(), align_of::<(usize, usize, T)>());

            let strong_ptr = ptr as *mut usize;
            let weak_ptr = ptr.add(size_of::<usize>()) as *mut usize;
            let value_ptr = ptr.add(size_of::<usize>() * 2) as *mut T;

            ptr::write(strong_ptr, 1);
            ptr::write(weak_ptr, 0);
            ptr::write(value_ptr, value);
            
            (strong_ptr, weak_ptr, value_ptr)
        };

        MyRc { strong_ptr, weak_ptr, value_ptr }
    }

    pub fn downgrade(&self) -> MyWeak<T> {
        unsafe {
            *self.weak_ptr += 1;
        }

        MyWeak { strong_ptr: self.strong_ptr, weak_ptr: self.weak_ptr, value_ptr: self.value_ptr }
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            & *self.value_ptr
        }
    }
}

impl<T> Display for MyRc<T>
where T: Display  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &**self)
    }
}

impl<T> Clone for MyRc<T> {    
    fn clone(&self) -> Self {
        unsafe {
            *self.strong_ptr += 1;
        }

        Self { strong_ptr: self.strong_ptr, weak_ptr: self.weak_ptr, value_ptr: self.value_ptr }
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        unsafe {
            *self.strong_ptr -= 1;
            if *self.strong_ptr == 0 {
                ptr::drop_in_place(self.value_ptr);
                if *self.weak_ptr == 0 {
                    my_free(self.strong_ptr);
                }
            }
        }
    }
}

pub struct MyWeak<T> {
    strong_ptr: *mut usize,
    weak_ptr: *mut usize,
    value_ptr: *mut T,
}

impl<T> MyWeak<T> {
    pub unsafe fn upgrade_unchecked(&self) -> MyRc<T> {
        unsafe {
            *self.strong_ptr += 1;
        }

        MyRc { strong_ptr: self.strong_ptr, weak_ptr: self.weak_ptr, value_ptr: self.value_ptr }
    }

    pub fn upgrade(&self) -> Option<MyRc<T>> {
        unsafe {
            if *self.strong_ptr != 0 {
                Some(self.upgrade_unchecked())
            } else {
                None
            }
        }
    }
}

impl<T> Clone for MyWeak<T> {
    fn clone(&self) -> Self {
        unsafe {
            *self.weak_ptr += 1;
        }

        Self { strong_ptr: self.strong_ptr, weak_ptr: self.weak_ptr, value_ptr: self.value_ptr }
    }
}

impl<T> Drop for MyWeak<T> {
    fn drop(&mut self) {
        unsafe {
            *self.weak_ptr -= 1;
            if *self.strong_ptr == 0 && *self.weak_ptr == 0 {
                my_free(self.strong_ptr);
            }
        }
    }
}
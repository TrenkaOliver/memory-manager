use std::{ops::{Deref, DerefMut}, ptr};

use crate::manager::{my_alloc, my_free};


pub struct MyRef<'a, T> {
    cell: &'a MyRefCell<T>
}

pub struct MyRefMut<'a, T> {
    cell: &'a MyRefCell<T>
}

pub struct MyRefCell<T> {
    state_ptr: *mut isize,
    value_ptr: *mut T,
}

impl<T> MyRefCell<T> {
    pub fn new(value: T) -> MyRefCell<T> {
        let (state_ptr, value_ptr) = unsafe {
            let ptr = my_alloc(size_of::<(isize, T)>(), align_of::<(isize, T)>());

            let state_ptr = ptr as *mut isize;
            let value_ptr = ptr.add(size_of::<isize>()) as *mut T;

            ptr::write(state_ptr, 0);
            ptr::write(value_ptr, value);

            (state_ptr, value_ptr)
        };

        MyRefCell { state_ptr, value_ptr }
    }

    pub fn borrow(&'_ self) -> MyRef<'_, T> {
        unsafe {
            if *self.state_ptr != -1 {
                *self.state_ptr += 1;
                MyRef { cell: &self }
            } else {
                panic!("already mutably borrowed: BorrowMutError")
            }
        }
    }

    pub fn borrow_mut(&'_ self) -> MyRefMut<'_, T> {
        unsafe {
            if *self.state_ptr == 0 {
                *self.state_ptr = -1;
                MyRefMut { cell: &self }
            } else {
                panic!("already borrowed: BorrowError")
            }
        }
    }
}

impl<T> Drop for MyRefCell<T> {
    fn drop(&mut self) {
        unsafe {
            my_free(self.state_ptr);
        }
    }    
}

impl<'a, T> Deref for MyRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            & *self.cell.value_ptr
        }
    }
}

impl<'a, T> Drop for MyRef<'a, T> {
    fn drop(&mut self) {
        unsafe {
            *self.cell.state_ptr -= 1;
        }
    }
}

impl<'a, T> Deref for MyRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            & *self.cell.value_ptr
        }
    }
}

impl<'a, T> DerefMut for MyRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.cell.value_ptr
        }
    }
}

impl<'a, T> Drop for MyRefMut<'a, T> {
    fn drop(&mut self) {
        unsafe {
            *self.cell.state_ptr = 0;
        }
    }
}
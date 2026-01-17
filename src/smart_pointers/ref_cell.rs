use core::{fmt::Display, ops::{Deref, DerefMut}};
use core::cell::UnsafeCell;

pub struct MyRef<'a, T> {
    ref_cell: &'a MyRefCell<T>
}

pub struct MyRefMut<'a, T> {
    ref_cell: &'a MyRefCell<T>
}

pub struct MyRefCell<T> {
    state: UnsafeCell<isize>,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for MyRefCell<T> {}

impl<T> MyRefCell<T> {
    pub fn new(value: T) -> MyRefCell<T> {
        MyRefCell { value: UnsafeCell::new(value), state: UnsafeCell::new(0) }
    }

    pub fn borrow(&'_ self) -> MyRef<'_, T> {
        let state = unsafe {
            &mut *self.state.get()
        };

        if *state != -1 {
            *state += 1;
            MyRef { ref_cell: &self }
        } else {
            panic!("already mutably borrowed: BorrowMutError")
        }
    }

    pub fn borrow_mut(&'_ self) -> MyRefMut<'_, T> {
        let state = unsafe {
            &mut *self.state.get()
        };

        if *state == 0 {
            *state = -1;
            MyRefMut { ref_cell: &self} 
        } else {
            panic!("already borrowed: BorrowError")
        }
    }
}

impl<'a, T> Deref for MyRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            & *self.ref_cell.value.get()
        }
    }
}

impl<'a, T> Drop for MyRef<'a, T> {
    fn drop(&mut self) {
        unsafe {
            *self.ref_cell.state.get() -= 1;
        }
    }
}

impl<'a, T: Display> Display for MyRef<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &**self)
    }
}

impl<'a, T> Deref for MyRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            & *self.ref_cell.value.get()
        }
    }
}

impl<'a, T> DerefMut for MyRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.ref_cell.value.get()
        }
    }
}

impl<'a, T> Drop for MyRefMut<'a, T> {
    fn drop(&mut self) {
        unsafe {
            *self.ref_cell.state.get() = 0;
        }
    }
}

impl<'a, T: Display> Display for MyRefMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &**self)
    }
}
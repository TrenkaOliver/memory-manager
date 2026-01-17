use core::sync::atomic::{AtomicBool, Ordering};
use core::ops::{Deref, DerefMut};
use core::fmt::Display;
use core::marker::PhantomData;

use crate::smart_pointers::unsafe_cell::MyUnsafeCell;

pub struct MyMutex<T> {
    locked: AtomicBool,
    value: MyUnsafeCell<T>,
}

unsafe impl<T: Send> Send for MyMutex<T> {}
unsafe impl<T: Send> Sync for MyMutex<T> {}

pub struct MyMutexGuard<'a, T> {
    mutex: &'a MyMutex<T>,
    _not_send: PhantomData<*const ()>
}

impl<T> MyMutex<T> {
    pub fn new(value: T) -> MyMutex<T> {
        MyMutex {
            locked: AtomicBool::new(false),
            value: MyUnsafeCell::new(value),
        }
    }

    pub fn lock(&'_ self) -> MyMutexGuard<'_, T> {
        while self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        };

        MyMutexGuard { mutex: &self, _not_send: PhantomData }
    }
}

impl<'a, T> Deref for MyMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            & *self.mutex.value.get()
        }
    }
}

impl<'a, T> DerefMut for MyMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *self.mutex.value.get()
        }
    }
}

impl<'a, T> Drop for MyMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}

impl<'a, T: Display> Display for MyMutexGuard<'a, T>  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &**self)
    }
}
use core::sync::atomic::{self, AtomicUsize, Ordering};
use core::ptr;
use std::fmt::Display;
use std::ops::Deref;

use crate::manager::{my_alloc, my_free};



pub struct MyArc<T> {
    inner: *mut MyArcInner<T>,
}

struct MyArcInner<T> {
    value: T,
    strong_count: AtomicUsize,
    weak_count: AtomicUsize,
}

unsafe impl<T: Send + Sync> Send for MyArc<T> {} 
unsafe impl<T: Send + Sync> Sync for MyArc<T> {} 

impl<T> MyArc<T> {
    pub fn new(value: T) -> MyArc<T> {
        let inner = my_alloc(size_of::<MyArcInner<T>>(), align_of::<MyArcInner<T>>()) as *mut MyArcInner<T>;

        let inner_value = MyArcInner {
            value, 
            strong_count: AtomicUsize::new(1), 
            weak_count: AtomicUsize::new(0)
        };

        unsafe {
            ptr::write(inner, inner_value);
        }

        MyArc { inner }
    }

    pub fn downgrade(&self) -> MyWeak<T> {
        unsafe {
            (*self.inner).weak_count.fetch_add(1, Ordering::Relaxed);
        }

        MyWeak { inner: self.inner }
    }

    unsafe fn get_inner_mut_ref(&self) -> &mut MyArcInner<T> {
        unsafe {
            &mut *self.inner
        }
    }
}

impl<T> Clone for MyArc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner).strong_count.fetch_add(1, Ordering::Relaxed);
        }

        Self { inner: self.inner }
    }
}

impl<T: Display> Display for MyArc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &**self)
    }
}

impl<T> Deref for MyArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &self.get_inner_mut_ref().value
        }
    }
}

impl<T> Drop for MyArc<T> {
    fn drop(&mut self) {
        let inner = unsafe {
            self.get_inner_mut_ref()
        };

        if inner.strong_count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        atomic::fence(Ordering::Acquire);

        unsafe {
            ptr::drop_in_place(&mut inner.value);
        }

        if inner.weak_count.load(Ordering::Acquire) != 0 {
            return;
        }

        unsafe {
            my_free(self.inner);
        }
    }
}

pub struct MyWeak<T> {
    inner: *mut MyArcInner<T>,
}

impl<T> MyWeak<T> {
    pub fn upgrade(&self) -> Option<MyArc<T>> {
        let inner = unsafe {
            &mut *self.inner
        };

        let mut current = inner.strong_count.load(Ordering::Acquire);

        loop {
            if current == 0 { return None; }
            match inner.strong_count.compare_exchange_weak(
                current, 
                current + 1,
                Ordering::Acquire,
                Ordering::Relaxed
            ) {
                Ok(_) => return Some(MyArc {inner: self.inner}),
                Err(prev) => current = prev,
            }
        }
    }
}

impl<T> Clone for MyWeak<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.inner).weak_count.fetch_add(1, Ordering::Relaxed);
        }

        Self { inner: self.inner }
    }
}

impl<T> Drop for MyWeak<T> {
    fn drop(&mut self) {
        let inner = unsafe {
            &mut *self.inner
        };

        if inner.weak_count.fetch_sub(1, Ordering::Release) == 1 &&
        inner.strong_count.load(Ordering::Acquire) == 0 {
            unsafe {
                my_free(self.inner);
            }
        }
    }
}

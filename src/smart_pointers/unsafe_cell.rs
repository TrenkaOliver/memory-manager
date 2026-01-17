use core::marker::PhantomData;

pub struct MyUnsafeCell<T> {
    value: T,
    _not_sync: PhantomData<*const ()>,
}

impl<T> MyUnsafeCell<T> {
    pub fn new(value: T) -> MyUnsafeCell<T> {
        MyUnsafeCell { value, _not_sync: PhantomData }
    }

    pub fn get(&self) -> *mut T {
        &raw const self.value as *mut T
    }
}
use core::marker::PhantomData;

pub struct MyUnsafeCell<T> {
    value: T,
    _not_send: PhantomData<*const ()>,
}

impl<T> MyUnsafeCell<T> {
    pub fn new(value: T) -> MyUnsafeCell<T> {
        MyUnsafeCell { value, _not_send: PhantomData }
    }

    pub fn get(&self) -> *mut T {
        &raw const self.value as *mut T
    }
}
mod manager;
mod collections;
mod smart_pointers;


//manager
pub use manager::debug_free;
pub use manager::my_alloc;
pub use manager::my_free;

//collections
pub use collections::string::MyString;
pub use collections::vec::*;

//refcell
pub use smart_pointers::*;
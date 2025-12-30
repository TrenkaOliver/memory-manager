const HEADER_SIZE: usize = size_of::<usize>() * 2;
const LEN: usize = 8192;

static mut HEAP: AlignedArray = AlignedArray::new();

static mut MANAGER: Manager = Manager::new();

pub fn debug_free() {
    unsafe {
        (* &raw const MANAGER).debug_free();
    }
}

//SAFETY:
//can only allocated sequentially, not thread safe
pub unsafe fn my_alloc(size: usize, alignment: usize) -> *mut u8 {
    unsafe {
        // MANAGER.alloc(size, alignment)
        (* &raw mut MANAGER).alloc(size, alignment)
    }
}

//SAFETY:
//can only free sequentially, not thread safe
//can only free ptr's given upon allocation
pub unsafe fn my_free<T>(ptr: *mut T) {
    unsafe {
        (* &raw mut MANAGER).free(ptr);
    }
}


//a wrapper around the bytes used as heap so it will always be 8 aligned
#[repr(align(8))]
struct AlignedArray([u8; LEN]);

impl AlignedArray {
    const fn new() -> AlignedArray {
        assert!(LEN > HEADER_SIZE);
        
        let mut bytes = [0u8; LEN];

        let size_bytes = LEN.to_ne_bytes();
        let max_bytes = usize::MAX.to_ne_bytes();

        let mut i = 0;
        while i < size_of::<usize>() {
            bytes[i] = size_bytes[i];
            bytes[i + size_of::<usize>()] = max_bytes[i];
            i+= 1;
        }

        AlignedArray(bytes)
    }
}

//first_free: points to the first free block
/*
    Free block layout:
    HEADER
        1. usize: size of block in bytes including HEADER
        2. *mut usize: points to the next free block
    REST OF BYTES
 */ 
/*
    ALlocated block layout:
    1. FRONT PADDING: enough bytes so the user data will have required alignment
    2. HEADER
        1. usize: size of the allocated block including everything
        2. *mut usize: points to the first byte of the allocated block
    3. USER DATA (bytes)
    4. END PADDING: max HEADER_SIZE bytes, so no bytes will be lost forever
 */
//Note: the last free block's ptr as usize == USIZE::MAX
struct Manager {
    first_free: *mut usize,
}

impl Manager {
    const fn new() -> Manager {
        let first_free = &raw mut HEAP as *mut usize;

        Manager { first_free }
    }

    //fn to debug free space, used for testing
    fn debug_free(&self) {
        println!("\ndebugging free sequences");
        println!("HEADER_SIZE = {}", HEADER_SIZE);
        let mut free_space = 0;
        let mut current = self.first_free;
        let mut i = 1;
        loop {
            let len = unsafe {
                & *current
            };
            println!("{}. free sequence len = {} bytes", i, len);
            free_space += len;

            current = unsafe {
                *current.add(1) as *mut usize
            };
            i += 1;

            if current as usize == usize::MAX {
                println!("end\n");
                break;
            }
        }
        println!("free space: {}", free_space);
    }

    unsafe fn alloc(&mut self, size: usize, alignment: usize) -> *mut u8 {
        //current_free is &mut to the pointer which points to the size (which is the first 4 bytes) of the currently inspected free block
        let mut current_free = &mut self.first_free;

        let alignment = alignment.max(8);

        let end_pad = (size_of::<usize>() - size % size_of::<usize>()) % size_of::<usize>();

        loop {
            //copy the pointee of current_size, so I don't need to dereference it later on each time I need the value
            let current_size = unsafe {
                **current_free
            };

            let mut front_pad = unsafe {
                alignment - (current_free.add(2) as usize % alignment)
            };

            if front_pad == alignment {
                front_pad = 0;
            }

            let new_size = front_pad + HEADER_SIZE + size + end_pad;

            //if it's (new_size) is larger, than check the next free block
            if new_size > current_size {
                current_free = unsafe {
                    &mut *(current_free.add(1) as *mut *mut usize)
                };

                //if this is the last block (ptr to the next free block as usize is the len of the bytes array), than panic
                if *current_free as usize == usize::MAX {
                    panic!("unable to allocate, not enough free space");
                }
            }
            //if the differrence between the current size and the new size is not greater than the header
            //than also append those extra bytes to the end of the new allocated block
            //bc it couldn't be used as a new free block, where later on new data could be allocated
            else if current_size - new_size <= HEADER_SIZE {
                let next_free = unsafe {
                    *current_free.add(1) as *mut usize
                };

                let front_pad = front_pad / 8;
                if front_pad != 0 {
                    unsafe {
                        *current_free.add(front_pad) = current_size;
                    }
                }

                unsafe {
                    *current_free.add(front_pad + 1) = *current_free as usize;
                }

                let ptr = unsafe {
                    current_free.add(2 + front_pad) as *mut u8
                };

                *current_free = next_free;

                return ptr;
            } 
            //if the diffrence is larger, than modify the current block
            else {
                //create a ptr to the place where i should slide the current free block
                let new_free = unsafe {
                    (*current_free as *mut u8).add(new_size) as *mut usize
                };

                //set the size the slided free block
                //set the slided free block's ptr to point the next free block
                unsafe {
                    *new_free = **current_free - new_size;
                    *new_free.add(1) = *current_free.add(1);
                }

                //for allocated block set size
                //for allocated block set ptr to the first byte (as *mut usize, ill need it as *mut usize for freeing)
                let front_pad = front_pad / 8;
                unsafe {
                    *current_free.add(front_pad) = new_size;
                    *current_free.add(front_pad + 1) = *current_free as usize
                }

                //create the output ptr
                let ptr = unsafe {
                    current_free.add(front_pad + 2) as *mut u8
                };

                //update the ptr to the current free block inside the previus free block to point to valid position
                *current_free = new_free;

                return ptr;
            }
        }
    }

    fn free<T>(&mut self, src: *mut T) {
        //sets the new free block's next free_block idx to the current first_free block's idx
        let ptr = src as *mut usize;

        let ptr_to_first_byte = unsafe {
            *ptr.sub(1) as *mut usize
        };

        let ptr_to_size = unsafe {
            ptr.sub(2)
        };

        unsafe {
            *ptr_to_first_byte = *ptr_to_size;
            *ptr_to_first_byte.add(1) = self.first_free as usize;
        }

        //sets the new free block's idx as the first_free block
        self.first_free = ptr_to_first_byte;
    }

}
const HEADER_SIZE: usize = size_of::<usize>() * 2;

pub struct Manager {
    first_free: *mut usize,
}

//IMPORTANT!!!
//Freeing or getting a random ptr (not one which was given upon allocation) is UB
//n := size_of::<usize>()
//free block: n bytes: size(usize), n bytes:ptr to next free block(*mut usize), the rest isn't used
//allocated block: n bytes: size(usize) n bytes: placeholder, ...used bytes..., max n bytes padding if necesarry
//first 2n bytes is the HEADER
//size includes the whole block not just the used/rest bytes
//the last free block's ptr to the next free block (which doesn't exists) as a usize is equal to the bytes array's len marking it as the last free block
//ptrs returned by the alloc fn will point to the first user used byte not the first in the header (size's first byte)
impl Manager {
    pub fn new(bytes: *mut [u8]) -> Manager {
        //creating manager for smaller array than HEADER_SIZE wouldn't make sense
        assert!(bytes.len() > HEADER_SIZE);
        
        unsafe {
            //sets the size of the first (currently only) free block to the len of the array
            *(bytes as *mut usize) = bytes.len();

            //as mentioned above, the ptr of the last free block as a usize is equal to the len of the byte array's len
            *(bytes as *mut usize).add(1) = usize::MAX;
        }

        //create a ptr to the first free block to know where to start searching for allocation
        let first_free = bytes as *mut usize;

        println!("starts at: {:?}", first_free as usize);

        Manager { first_free }
    }

    //fn to debug free space, used for testing
    pub fn debug_free(&self) {
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

    pub fn alloc(&mut self, size: usize, alignment: usize) -> *mut u8 {
        //current_free is &mut to the pointer which points to the size (which is the first 4 bytes) of the currently inspected free block
        let mut current_free = &mut self.first_free;

        let alignment = alignment.max(8);

        let mut end_pad = size_of::<usize>() - size % size_of::<usize>();
        if end_pad == size_of::<usize>() {
            end_pad = 0;
        }

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

            //if it's (data + header size) is larger, than check the next free block
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
                unsafe {
                    *new_free = **current_free - new_size;
                }

                //set the slided free block's ptr to point the next free block
                unsafe {
                    *new_free.add(1) = *current_free.add(1);
                }

                let front_pad = front_pad / 8;
                //for allocated block set size
                unsafe {
                    *current_free.add(front_pad) = new_size;
                }

                unsafe {
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

    pub fn free<T>(&mut self, src: *mut T) {
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
        }

        unsafe {
            *ptr_to_first_byte.add(1) = self.first_free as usize;
        }

        //sets the new free block's idx as the first_free block
        self.first_free = ptr_to_first_byte;
    }

}
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use std::vec::Vec;


pub struct Arena<T>
{
	chunk_size : usize,
	heap : Vec<Vec<Option<T>>>,
	freed : Vec<usize>,
}

impl<T> Arena<T> 
{	
	pub fn new(chunk_size : usize) -> Self 
	{		
		let mut heap = Vec::new();
		heap.push(Vec::new());

		Self 
		{
			chunk_size : chunk_size,
			heap : heap,
			freed : Vec::new(),	
		}
	}

	pub fn alloc(&mut self, obj: T) -> usize
	{		
		//Chunk is full, need to alloc new chunk.
		if self.heap.last().unwrap().len() == self.chunk_size 
		{
			self.heap.push(Vec::new());
			let current_age = self.heap.len() - 1;
			self.heap[current_age].reserve(self.chunk_size);
		}		

		if self.freed.len() == 0  
		{
			let current_age = self.heap.len() - 1;
			self.heap[current_age].push(Some(obj));
			let index = current_age * self.chunk_size + self.heap[current_age].len() - 1;
			index
		}
		else 
		{
			let index = self.freed.pop().unwrap();
			self.heap[index / self.chunk_size][index % self.chunk_size] = Some(obj);
			index
		}
	}

	pub fn len(&self) -> usize
	{
		(self.heap.len() - 1) * self.chunk_size + self.heap.last().unwrap().len()
	}

	fn check_index(&self, index : usize) -> bool
	{		
		let age = index / self.chunk_size;
	 	let last_index = index % self.chunk_size;

		if age < self.heap.len() 
			&& last_index < self.heap[age].len()
		{
			true
		}
		else
		{
			false
		}
	}	

	pub fn free(&mut self, index : usize) 
	{
		if self.check_index(index) == false
		{
			panic!("Wrong Arena index = {} for deleting !", index)
		}

		if self.heap[index / self.chunk_size][index % self.chunk_size].is_none()
		{
			panic!("Attempt to free deleted object with index = {}.", index)
		}

		self.heap[index / self.chunk_size][index % self.chunk_size].take().unwrap();
		self.freed.push(index);
	}

	pub fn exist(&self, index : usize) -> bool
	{
		if self.check_index(index) == true && self.heap[index / self.chunk_size][index % self.chunk_size].is_some()
		{
			true
		}
		else
		{
			false
		}
	}

	pub fn is_freed(&self, index : usize) -> bool
	{
		if self.check_index(index) == true && self.heap[index / self.chunk_size][index % self.chunk_size].is_none()
		{
			true
		}
		else
		{
			false
		}
	}
}

impl<T> std::ops::Index<usize> for Arena<T> 
{
    type Output = Option<T>;

    fn index(&self, index : usize) -> &Self::Output 
    {
		if self.check_index(index) == false
    	{
    		panic!("Invalid index = {} for Arena !", index)
    	}

    	&self.heap[index / self.chunk_size][index % self.chunk_size]
    }
}


#[cfg(test)]
mod tests 
{
	use super::*;
    use crate::arena::{Arena};
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{thread, time};


    const TEST_ARENA_CHUNK_SIZE : usize = 64;

    #[derive(Debug)]
    struct MyStruct
    {
    	x : usize,
    	y : String,
    }

    impl MyStruct
    {
    	pub fn new(x : usize, y : &str) -> Self
    	{
    		MyStruct
    		{
    			x : x,
    			y : y.to_string(), 
    		}
    	}
    }

	impl PartialEq for MyStruct 
	{
    	fn eq(&self, other: &Self) -> bool 
    	{
    		if self.x == other.x && self.y == other.y
    		{
    			true
    		}
    		else 
    		{
    			false
    		}
    	}
	}

	fn rand(max : usize) -> usize
	{
    	let nanos = SystemTime::now()
        	.duration_since(UNIX_EPOCH)
        	.unwrap()
        	.subsec_nanos() as usize;

    	nanos / 100 % max
    }

	//Alloc 'n' objects in a new Arena
	//For more test accuracy need MyStruct::new(i,"All is fine")
	fn arena_alloc_n(n : usize) -> (Arena<MyStruct>, Vec<usize>)
	{
        let mut arena = Arena::<MyStruct>::new(TEST_ARENA_CHUNK_SIZE);
        let mut indexes = Vec::new();

        for i in 0..n
        {
        	//For more test accuracy need MyStruct::new(i,"All is fine")
        	indexes.push(arena.alloc(MyStruct::new(i, "All is fine")));
        }

        (arena, indexes)
	}

    #[test]
    fn arena_new() 
    {
        let arena = Arena::<MyStruct>::new(TEST_ARENA_CHUNK_SIZE);

        assert_eq!(arena.len(), 0);
        assert_eq!(arena.chunk_size, TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.freed.len(), 0);
    }

   #[test]
    fn arena_alloc() 
    {
        let mut arena = Arena::<MyStruct>::new(TEST_ARENA_CHUNK_SIZE);
        let index = arena.alloc(MyStruct::new(16838, "All is fine"));

        assert_eq!(arena.len(), 1);
        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.heap[0].len(), 1);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(index, 0);
    }     

    #[test]
    fn arena_alloc5() 
    {
        let mut arena = Arena::<MyStruct>::new(TEST_ARENA_CHUNK_SIZE);        

        let index0 = arena.alloc(MyStruct::new(0, "All is fine 0"));
        let index1 = arena.alloc(MyStruct::new(1, "All is fine 1"));
        let index2 = arena.alloc(MyStruct::new(2, "All is fine 2"));
        let index3 = arena.alloc(MyStruct::new(3, "All is fine 3"));
        let index4 = arena.alloc(MyStruct::new(4, "All is fine 4"));
        assert_eq!(arena.len(), 5);

        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.heap[0].len(), 5);
        assert_eq!(arena.freed.len(), 0);

        assert_eq!(arena[index0] , Some(MyStruct::new(0, "All is fine 0")));
        assert_eq!(index0, 0);

        assert_eq!(arena[index1] , Some(MyStruct::new(1, "All is fine 1")));
        assert_eq!(index1, 1);
        
        assert_eq!(arena[index2] , Some(MyStruct::new(2, "All is fine 2")));
        assert_eq!(index2, 2);
        
        assert_eq!(arena[index3] , Some(MyStruct::new(3, "All is fine 3")));
        assert_eq!(index3, 3);
        
        assert_eq!(arena[index4] , Some(MyStruct::new(4, "All is fine 4")));
        assert_eq!(index4, 4);
	}         

    #[test]
    fn arena_index() 
    {
    	//We force to alloc new chunk
        let (arena, indexs) = arena_alloc_n(TEST_ARENA_CHUNK_SIZE + 1);
        assert_eq!(arena.len(), TEST_ARENA_CHUNK_SIZE + 1);

        assert_eq!(arena.heap.len(), 2);//Two chunks in the heap
        assert_eq!(arena.heap[0].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[1].len(), 1);        
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(indexs[0], 0);
        assert_eq!(indexs[1], 1);
        assert_eq!(indexs[2], 2);
        assert_eq!(indexs[3], 3);
        assert_eq!(indexs[4], 4);
        assert_eq!(indexs[TEST_ARENA_CHUNK_SIZE - 1] , TEST_ARENA_CHUNK_SIZE - 1);
        assert_eq!(indexs[TEST_ARENA_CHUNK_SIZE] , TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.check_index(TEST_ARENA_CHUNK_SIZE + 1) , false);
    }                     

    #[test]
    fn arena_alloc_check_index() 
    {
    	//We force to alloc new chunk, with one element
        let (arena, indexs) = arena_alloc_n(TEST_ARENA_CHUNK_SIZE + 1);
        assert_eq!(arena.len(), TEST_ARENA_CHUNK_SIZE + 1);

        assert_eq!(arena.heap.len(), 2);//Two chunks in a heap
        assert_eq!(arena.heap[0].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[1].len(), 1);
        assert_eq!(arena.freed.len(), 0);

        let first0 = 0;
        let last0 = TEST_ARENA_CHUNK_SIZE - 1;
        let first1 = TEST_ARENA_CHUNK_SIZE;
        let last1 = TEST_ARENA_CHUNK_SIZE + 1;
        let fake_index = 1000;

        assert_eq!(arena.check_index(first0), true);
        assert_eq!(arena.check_index(last0), true);
        assert_eq!(arena.check_index(first1), true);
        assert_eq!(arena.check_index(last1), false);
        assert_eq!(arena.check_index(fake_index), false);

        //Check indexes
        let mut index = 0;
        while index < TEST_ARENA_CHUNK_SIZE+1
        {
        	assert_eq!(indexs[index], index);
        	index += 1;
        }        
    }             

    #[test]
    fn arena_free() 
    {
		let (mut arena, indexs) = arena_alloc_n(100 * TEST_ARENA_CHUNK_SIZE + 1);
		assert_eq!(arena.len(), 100 * TEST_ARENA_CHUNK_SIZE + 1);

        assert_eq!(arena.heap.len(), 101);
        assert_eq!(arena.heap[0].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[1].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[2].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.freed.len(), 0);

		//Check indexes
        let mut index = 0;
        while index < 100 * TEST_ARENA_CHUNK_SIZE+1
        {
        	assert_eq!(indexs[index], index);
        	index += 1;
        }        

        let mut rands = HashMap::new();

        while rands.len() < 3000
        {
        	let r = rand(6000);

        	if rands.get(&r) == None
        	{
        		rands.insert(r, false);
        	}
        }

    	for (key, value) in &mut rands
    	{
    		assert_eq!(*value, false);
    		arena.free(*key);
    		*value = true;
	    }	   

	    let mut freed : usize = 0;
	    for i in 0..(arena.len()-1)
	    {
	    	if arena.is_freed(i) && rands[&i] == true
	    	{
	    		freed += 1;
	    	}
	    }

	    assert_eq!(rands.len(), freed);
	}


    #[test]
    fn arena_free_and_alloc_after_free() 
    {
		let (mut arena, indexs) = arena_alloc_n(100 * TEST_ARENA_CHUNK_SIZE + 1);
		assert_eq!(arena.len(), 100 * TEST_ARENA_CHUNK_SIZE + 1);

        assert_eq!(arena.heap.len(), 101);
        assert_eq!(arena.heap[0].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[1].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[2].len(), TEST_ARENA_CHUNK_SIZE);        
        assert_eq!(arena.freed.len(), 0);

        let age = 13;
        let index = 13;
		let index1 = age * arena.chunk_size + index;// 845
		arena.free(index1);
		assert_eq!(arena.freed.len(), 1);
		assert_eq!(arena.freed[0], index1);		

		assert_eq!(arena[index1 - 1], Some(MyStruct::new(index1 - 1, "All is fine")));
		assert_eq!(arena[index1], None);
		assert_eq!(arena[index1 + 1], Some(MyStruct::new(index1 + 1, "All is fine")));

        let age = 99;
        let index = 63;
		let index2 = age * arena.chunk_size + index;// 6399
		arena.free(index2);
		assert_eq!(arena.freed.len(), 2);
		assert_eq!(arena.freed[1], index2);		

		assert_eq!(arena[index2 - 1], Some(MyStruct::new(6398, "All is fine")));
		assert_eq!(arena[index2], None);
		assert_eq!(arena[index2 + 1], Some(MyStruct::new(6400, "All is fine")));

		//alloc after free
		let new_index1 = arena.alloc(MyStruct::new(777, "All is fine"));
		assert_eq!(index2, new_index1);
		assert_eq!(arena[index2], Some(MyStruct::new(777, "All is fine")));		
		assert_ne!(arena[index2].as_ref().unwrap().x, 776);
		assert_eq!(arena[index2].as_ref().unwrap().x, 777);
		assert_ne!(arena[index2].as_ref().unwrap().x, 778);
		assert_eq!(arena[index2].as_ref().unwrap().y, "All is fine");
		assert_eq!(arena.freed.len(), 1);

		let new_index2 = arena.alloc(MyStruct::new(888, "All is fine"));
		assert_eq!(index1, new_index2);
		assert_eq!(arena[index1], Some(MyStruct::new(888, "All is fine")));
		assert_ne!(arena[index2].as_ref().unwrap().x, 887);
		assert_eq!(arena[index1].as_ref().unwrap().x, 888);
		assert_ne!(arena[index2].as_ref().unwrap().x, 889);
		assert_eq!(arena[index1].as_ref().unwrap().y, "All is fine");
		assert_eq!(arena.freed.len(), 0);

		//Check indexes
        let mut index = 0;
        while index < 100 * TEST_ARENA_CHUNK_SIZE+1
        {
        	assert_eq!(indexs[index], index);
        	index += 1;
        }
    }	

    // #[test]
    // fn arena_rand_alloc_free_alloc_after_after() 
    // {
	// 	static : Type = init;	
    // }        
     
}//mod tests
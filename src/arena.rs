// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]

use std::vec::Vec;
use std::sync::Mutex;


static ARENA_ID : Mutex<usize> = Mutex::new(0);

pub struct Arena<T>
{
	id : usize,
	chunk_size : usize,
	heap : Vec<Vec<Option<T>>>,
	freed : Vec<usize>,
}

impl<T> Arena<T> 
{	
	pub fn new(chunk_size : usize) -> Self 
	{		
		let arena_id : usize =		
		{
			let mut id = ARENA_ID.lock().unwrap();
			*id += 1;
			*id
		};

		let mut heap = Vec::new();
		heap.push(Vec::new());

		Self 
		{
			id : arena_id,
			chunk_size : chunk_size,
			heap : heap,
			freed : Vec::new(),	
		}
	}

	pub fn id(&self) -> usize
	{
		self.id
	}

	pub fn alloc(&mut self, obj: T) -> (usize, usize)//(index, arena_id)
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
			(index, self.id)
		}
		else 
		{
			let index = self.freed.pop().unwrap();
			self.heap[index / self.chunk_size][index % self.chunk_size] = Some(obj);
			(index, self.id)
		}
	}

	pub fn get_index(&self, age : usize, index : usize) -> usize
	{
		age * self.chunk_size + index
	}

	pub fn get_index_pair(&self, index : usize) -> (usize, usize)
	{
		let age = index / self.chunk_size;
		let last_index = 
		{
			if index > self.chunk_size
			{
				index % self.chunk_size
			}
			else
			{
				index
			}
		};

		(age, last_index)
	}

	fn check_index(&self, index : (usize, usize)) -> bool
	{		
		let pair = self.get_index_pair(index.0);

		println!("Index age = {:?}", pair.0);
		println!("Index index = {:?}", pair.1);
		println!(" ");

		if self.id == index.1
			&& pair.0 < self.heap.len()
				&& pair.1 < self.heap[pair.0].len()
		{
			true
		}
		else
		{
			false
		}
	}	

	pub fn free(&mut self, index : (usize, usize)) 
	{
		if self.check_index(index) == false && self[index].is_some() == false
		{
			panic!("Wrong Arena index for freeing !")
		}

		self.heap[index.0 / self.chunk_size][index.0 % self.chunk_size].take().unwrap();
		self.freed.push(index.0);
	}	
}

impl<T> std::ops::Index<(usize, usize)> for Arena<T> 
{
    type Output = Option<T>;

    fn index(&self, index : (usize, usize)) -> &Self::Output 
    {
		if self.check_index(index) == false
    	{
    		panic!("Invalid index for Arena !")
    	}

    	&self.heap[index.0 / self.chunk_size][index.0 % self.chunk_size]
    }
}


#[cfg(test)]
mod tests 
{
	use super::*;
    use crate::arena::{Arena};

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

    #[test]
    fn arena_new() 
    {
        let arena = Arena::<MyStruct>::new(TEST_ARENA_CHUNK_SIZE);

        assert_eq!(arena.chunk_size, TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.freed.len(), 0);
    }

   #[test]
    fn arena_alloc() 
    {
        let mut arena = Arena::<MyStruct>::new(TEST_ARENA_CHUNK_SIZE);
        let index = arena.alloc(MyStruct::new(16838, "All is fine"));

        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.heap[0].len(), 1);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(index.0, 0);
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

        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.heap[0].len(), 5);
        assert_eq!(arena.freed.len(), 0);

        assert_eq!(arena[index0] , Some(MyStruct::new(0, "All is fine 0")));
        assert_eq!(index0.0, 0);

        assert_eq!(arena[index1] , Some(MyStruct::new(1, "All is fine 1")));
        assert_eq!(index1.0, 1);
        
        assert_eq!(arena[index2] , Some(MyStruct::new(2, "All is fine 2")));
        assert_eq!(index2.0, 2);
        
        assert_eq!(arena[index3] , Some(MyStruct::new(3, "All is fine 3")));
        assert_eq!(index3.0, 3);
        
        assert_eq!(arena[index4] , Some(MyStruct::new(4, "All is fine 4")));
        assert_eq!(index4.0, 4);
	}         

	//Alloc 'n' objects in a new Arena
	//For more test accuracy need MyStruct::new(i,"All is fine")
	fn arena_alloc_n(n : usize) -> (Arena<MyStruct>, Vec<(usize, usize)>)
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
    fn arena_index() 
    {
    	//We force to alloc new chunk
        let (arena, indexs) = arena_alloc_n(TEST_ARENA_CHUNK_SIZE + 1);

        assert_eq!(arena.heap.len(), 2);//Two chunks in the heap
        assert_eq!(arena.heap[0].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[1].len(), 1);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(indexs[0].0, 0);
        assert_eq!(indexs[1].0, 1);
        assert_eq!(indexs[2].0, 2);
        assert_eq!(indexs[3].0, 3);
        assert_eq!(indexs[4].0, 4);
        assert_eq!(indexs[TEST_ARENA_CHUNK_SIZE - 1].0 , TEST_ARENA_CHUNK_SIZE - 1);
        assert_eq!(indexs[TEST_ARENA_CHUNK_SIZE].0 , TEST_ARENA_CHUNK_SIZE);
    }             

    #[test]
    fn arena_alloc_chunk_size() 
    {
    	//We force to alloc new chunk
        let (arena, indexs) = arena_alloc_n(TEST_ARENA_CHUNK_SIZE + 1);

        assert_eq!(arena.heap.len(), 2);//Two chunks in the heap
        assert_eq!(arena.heap[0].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[1].len(), 1);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(indexs.len(), TEST_ARENA_CHUNK_SIZE + 1);  
        assert_eq!(indexs[TEST_ARENA_CHUNK_SIZE - 1].0 , TEST_ARENA_CHUNK_SIZE - 1);
        assert_eq!(indexs[TEST_ARENA_CHUNK_SIZE].0 , TEST_ARENA_CHUNK_SIZE);
    }             

    #[test]
    fn arena_alloc_check_index() 
    {
    	//We force to alloc new chunk, with one element
        let (arena, indexs) = arena_alloc_n(TEST_ARENA_CHUNK_SIZE + 1);

        assert_eq!(arena.heap.len(), 2);//Two chunks in a heap
        assert_eq!(arena.heap[0].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.heap[1].len(), 1);
        assert_eq!(arena.freed.len(), 0);

        let first0 = (0, arena.id());
        let last0 = (TEST_ARENA_CHUNK_SIZE - 1, arena.id());
        let after_last0 = (TEST_ARENA_CHUNK_SIZE, arena.id());
        let fake_index = (0, 333);

        assert_eq!(arena.check_index(first0), true);
        assert_eq!(arena.check_index(last0), true);
        assert_eq!(arena.check_index(after_last0), false);
        assert_eq!(arena.check_index(fake_index), false);

        //Check indexes
        let mut index = 0;
        for i in 0..TEST_ARENA_CHUNK_SIZE+1
        {
        	assert_eq!(indexs[i].0, index);

        	index += 1;
        }        
    }             

    #[test]
    fn arena_free_and_alloc_after_free() 
    {
		let (mut arena, indexs) = arena_alloc_n(100 * TEST_ARENA_CHUNK_SIZE + 1);

        assert_eq!(arena.heap.len(), 101);
        assert_eq!(arena.heap[0].len(), TEST_ARENA_CHUNK_SIZE);
        assert_eq!(arena.freed.len(), 0);

		let index1 = (arena.get_index(13, 13), arena.id());
		arena.free(index1);
		assert_eq!(arena.freed.len(), 1);
		assert_eq!(arena.freed[0], index1.0);		

		assert_eq!(arena.heap[13][12], Some(MyStruct::new(13*TEST_ARENA_CHUNK_SIZE+12, "All is fine")));
		assert_eq!(arena[index1], None);
		assert_eq!(arena.heap[13][14], Some(MyStruct::new(13*TEST_ARENA_CHUNK_SIZE+14, "All is fine")));

		//let index2 = Index{arena_id : arena.id(), age : 100, index : 0};
		let index2 = (arena.get_index(100, 0), arena.id());
		arena.free(index2);
		assert_eq!(arena.freed.len(), 2);
		assert_eq!(arena.freed[1], index2.0);		

		assert_eq!(arena.heap[99][TEST_ARENA_CHUNK_SIZE - 1], Some(MyStruct::new(99*TEST_ARENA_CHUNK_SIZE+63, "All is fine")));
		assert_eq!(arena[index2], None);

		//alloc after free
		let new_index1 = arena.alloc(MyStruct::new(777, "All is fine"));
		assert_eq!(index2, new_index1);
		assert_eq!(arena[index2], Some(MyStruct::new(777, "All is fine")));
		assert_eq!(arena.freed.len(), 1);

		let new_index2 = arena.alloc(MyStruct::new(888, "All is fine"));
		assert_eq!(index1, new_index2);
		assert_eq!(arena[index1], Some(MyStruct::new(888, "All is fine")));		
		assert_eq!(arena.freed.len(), 0);

		//Check indexes
        let mut index = 0;
        for i in 0..TEST_ARENA_CHUNK_SIZE+1
        {
        	assert_eq!(indexs[i].0, index);

        	index += 1;
        }  		
    }         
}//mod tests
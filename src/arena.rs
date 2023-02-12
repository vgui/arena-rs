#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::vec::Vec;


const ARENA_CHUNCK_SIZE : usize = 64;
pub const ARENA_NO_INDEX : usize = usize::MAX;

#[derive(Copy, Clone)]
pub struct ArenaIndex 
{
	age : usize,
	index : usize,
}

impl ArenaIndex
{
	pub fn new() -> Self
	{
		ArenaIndex
		{
			age : ARENA_NO_INDEX,
			index : ARENA_CHUNCK_SIZE,
		}
	}

	pub fn is_valid(&self) -> bool
	{
		if self.age != ARENA_NO_INDEX && self.index != ARENA_CHUNCK_SIZE
		{
			true
		}
		else
		{
			false
		}
	}
}

pub struct Arena<T>
{
	heap : Vec<Vec<Option<T>>>,
	freed : Vec<ArenaIndex>,
	//next : ArenaIndex,
	current_age : usize,
	next_index : usize,
}

impl<T> Arena<T> 
{	
	pub fn new() -> Self 
	{
		Self 
		{
			heap : Vec::new(),
			freed : Vec::new(),
			//next : ArenaIndex::new(),
			current_age : ARENA_NO_INDEX,
			next_index : ARENA_CHUNCK_SIZE,			
		}
	}

	fn alloc_chunk(&mut self) -> ArenaIndex 
	{
		if self.next_index == ARENA_CHUNCK_SIZE 
		{
			self.heap.push(Vec::new());		
			self.next_index = 0;
		
			if self.current_age == ARENA_NO_INDEX 
			{
				self.current_age = 0;
			}
			else
			{
				self.current_age += 1;
			}		
		}	

		self.heap[self.current_age].reserve(ARENA_CHUNCK_SIZE);

		ArenaIndex {age : self.current_age, index : self.next_index,}		
	}

	pub fn alloc(&mut self, obj: T) -> ArenaIndex		
	{
		self.alloc_chunk();

		if self.freed.len() == 0  
		{
			self.heap[self.current_age].push(Some(obj));
			let index = ArenaIndex {age : self.current_age, index : self.next_index,};
			self.next_index += 1;					
			index
		}
		else 
		{
			let index = self.freed.pop().unwrap();
			self.heap[index.age][index.index] = Some(obj);
			index
		}
	}

	fn check_index(&self, index : ArenaIndex) -> bool
	{		
		if self.heap.len() > index.age && self.heap[index.age].len() > index.index
		{
			true
		}
		else
		{
			false
		}
	}	

	pub fn free(&mut self, index : ArenaIndex) 
	{
		if index.is_valid() && self.check_index(index) == false
		{
			panic!("Wrong Arena index for freeing !")
		}

		self.heap[index.age][index.index].take().unwrap();
		self.freed.push(index);
	}	
}

impl<T> std::ops::Index<ArenaIndex> for Arena<T> 
{
    type Output = Option<T>;

    fn index(&self, index : ArenaIndex) -> &Self::Output 
    {
		if self.check_index(index) == false
    	{
    		panic!("Invalid index for Arena !")
    	}

    	&self.heap[index.age][index.index]
    }
}


#[cfg(test)]
mod tests 
{
	use super::*;
    use crate::arena::{ArenaIndex, Arena};

    struct MyStruct;

    #[test]
    fn arena_new() 
    {
        let arena = Arena::<MyStruct>::new();

        assert_eq!(arena.heap.len(), 0);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(arena.current_age, ARENA_NO_INDEX);
        assert_eq!(arena.next_index, ARENA_CHUNCK_SIZE);        
    }

   #[test]
    fn arena_alloc_chunck() 
    {
        let mut arena = Arena::<MyStruct>::new();
        let index = arena.alloc_chunk();

        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(arena.current_age, 0);
        assert_eq!(arena.next_index, 0);  
    }    

   #[test]
    fn arena_alloc() 
    {
        let mut arena = Arena::<MyStruct>::new();
        let index = arena.alloc(MyStruct);

        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.heap[0].len(), 1);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(arena.current_age, 0);
        assert_eq!(arena.next_index, 1);  
    }     

    #[test]
    fn arena_alloc5() 
    {
        let mut arena = Arena::<MyStruct>::new();
        let index = arena.alloc(MyStruct);
        let index = arena.alloc(MyStruct);
        let index = arena.alloc(MyStruct);
        let index = arena.alloc(MyStruct);
        let index = arena.alloc(MyStruct);

        assert_eq!(arena.heap.len(), 1);
        assert_eq!(arena.heap[0].len(), 5);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(arena.current_age, 0);
        assert_eq!(arena.next_index, 5);  
    }         

    #[test]
    fn arena_alloc_chunck_size() 
    {
        let mut arena = Arena::<MyStruct>::new();

        for i in 0..ARENA_CHUNCK_SIZE+1//[start..end-1]
        {
        	arena.alloc(MyStruct);
        }

        assert_eq!(arena.heap.len(), 2);
        assert_eq!(arena.heap[0].len(), ARENA_CHUNCK_SIZE);
        assert_eq!(arena.heap[1].len(), 1);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(arena.current_age, 1);
        assert_eq!(arena.next_index, 1);  
    }             

    #[test]
    fn arena_alloc_check_index() 
    {
        let mut arena = Arena::<MyStruct>::new();

        for i in 0..ARENA_CHUNCK_SIZE+1//[start..end-1]
        {
        	arena.alloc(MyStruct);
        }

        assert_eq!(arena.heap.len(), 2);
        assert_eq!(arena.heap[0].len(), ARENA_CHUNCK_SIZE);
        assert_eq!(arena.heap[1].len(), 1);
        assert_eq!(arena.freed.len(), 0);
        assert_eq!(arena.current_age, 1);
        assert_eq!(arena.next_index, 1);  

        let first0 = ArenaIndex{age : 0, index : 0};
        let last0 = ArenaIndex{age : 0, index : ARENA_CHUNCK_SIZE - 1};
        let after_last0 = ArenaIndex{age : 0, index : ARENA_CHUNCK_SIZE};

        let first1 = ArenaIndex{age : 1, index : 0};
        let last1 = ArenaIndex{age : 1, index : 0};
        let after_last1 = ArenaIndex{age : 1, index : 1};

        assert_eq!(arena.check_index(first0), true);
        assert_eq!(arena.check_index(last0), true);
        assert_eq!(arena.check_index(after_last0), false);

        assert_eq!(arena.check_index(first1), true);
        assert_eq!(arena.check_index(last1), true);
        assert_eq!(arena.check_index(after_last1), false);        
    }             

}
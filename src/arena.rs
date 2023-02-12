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
	next : ArenaIndex,
}

impl<T> Arena<T> 
{
	
	pub fn new() -> Self 
	{
		Self 
		{
			heap : Vec::new(),
			freed : Vec::new(),
			next : ArenaIndex::new(),
		}
	}

	fn alloc_chank(&mut self) -> ArenaIndex 
	{
		if self.next.index == ARENA_CHUNCK_SIZE 
		{
			self.heap.push(Vec::new());		
			self.next.index = 0;
		
			if self.next.age == ARENA_NO_INDEX 
			{
				self.next.age = 0;
			}
			else
			{
				self.next.age += 1;
			}		
		}	

		self.heap[self.next.age].reserve(ARENA_CHUNCK_SIZE);
		self.next
	}

	pub fn alloc(&mut self, obj: T) -> ArenaIndex		
	{
		self.alloc_chank();

		if self.freed.len() == 0  
		{
			self.heap[self.next.age].push(Some(obj));
			let index = self.next;
			self.next.index += 1;					
			index
		}
		else 
		{
			let index = self.freed.pop().unwrap();
			self.heap[index.age][index.index] = Some(obj);
			index
		}
	}

	pub fn free(&mut self, index : ArenaIndex) 
	{
		if self.check_index(index) == false
		{
			panic!("Wrong Arena index for freeing !")
		}

		self.heap[index.age][index.index].take().unwrap();
		self.freed.push(index);
	}

	fn check_index(&self, index : ArenaIndex) -> bool
	{		
		if index.is_valid() && self.heap.len() < index.age &&
		   self.heap[index.age].len() < index.index
		{
			true
		}
		else
		{
			false
		}
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
#![allow(dead_code)]

use jelly_mem_access::mem_accessor::MemAccess;
use jelly_mem_access::UioAccessor;
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;

struct Accessor {
    id: i32,
    map: HashMap<i32, (Box<dyn MemAccess>, usize)>,
}

impl Accessor {
    pub fn new() -> Self {
        Accessor {
            id: 1,
            map: HashMap::new(),
        }
    }

    pub fn open_uio(&mut self, name: &str, unit: usize) -> Result<i32, Box<dyn Error>> {
        let accessor = Box::new(UioAccessor::<u8>::new_with_name(name)?);

        let id = self.id;
        self.map.insert(id, (accessor, unit));
        self.id += 1;

        Ok(id)
    }

//    pub fn subclone(&mut self, name: &str, unit: i32) -> Result<i32, Box<dyn Error>> {
//    }

    pub fn close(&mut self, id: i32) -> Result<(), Box<dyn Error>> {
        self.map.remove(&id).ok_or("Invalid id")?;
        Ok(())
    }

    fn accessor(&self, id: i32) -> Result<(&Box<dyn MemAccess>, usize), Box<dyn Error>> {
        let (accessor, unit) = self.map.get(&id).ok_or("Invalid id")?;
        Ok((accessor, *unit))
    }

    pub fn write_mem(&self, id: i32, offset: usize, data: usize) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem(offset, data);
        }
        Ok(())
    }

    pub fn write_mem_u8(&self, id: i32, offset: usize, data: u8) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u8(offset, data);
        }
        Ok(())
    }

    pub fn write_reg_u8(&self, id: i32, reg: usize, data: u8) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u8(reg * unit, data);
        }
        Ok(())
    }

    pub fn write_mem_u16(
        &mut self,
        id: i32,
        offset: usize,
        data: u16,
    ) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u16(offset, data);
        }
        Ok(())
    }

    pub fn write_mem_u32(
        &mut self,
        id: i32,
        offset: usize,
        data: u32,
    ) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u32(offset, data);
        }
        Ok(())
    }

    pub fn write_mem_u64(
        &mut self,
        id: i32,
        offset: usize,
        data: u64,
    ) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u64(offset, data);
        }
        Ok(())
    }
}

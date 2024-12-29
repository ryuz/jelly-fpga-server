#![allow(dead_code)]

use jelly_mem_access::mem_accessor::MemAccess;
use jelly_mem_access::UioAccessor;
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;
use std::any::Any;

type Id = u32;

struct Accessor {
    id: Id,
    map: HashMap<Id, (Box<dyn MemAccess>, usize)>,
}

impl Accessor {
    pub fn new() -> Self {
        Accessor {
            id: 1,
            map: HashMap::new(),
        }
    }

    pub fn open_uio(&mut self, name: &str, unit: usize) -> Result<Id, Box<dyn Error>> {
        let accessor = Box::new(UioAccessor::<u8>::new_with_name(name)?);

        let id = self.id;
        self.map.insert(id, (accessor, unit));
        self.id += 1;

        Ok(id)
    }

    fn accessor(&self, id: Id) -> Result<(&Box<dyn MemAccess>, usize), Box<dyn Error>> {
        let (accessor, unit) = self.map.get(&id).ok_or("Invalid id")?;
        Ok((accessor, *unit))
    }

    /*
    pub fn subclone(&mut self, id: Id, offset: usize, size: usize, unit: usize) -> Result<Id, Box<dyn Error>> {
        let (accessor, unit_org) = self.accessor(id)?;
        let unit = if unit == 0 { unit_org } else { unit };
        if let Some(&acc) = accessor.as_any().downcast_ref::<UioAccessor<u8>>() {
            let accessor = acc.subclone::<u8>(offset, size)?;
            let id = self.id;
            self.map.insert(id, (accessor, unit));
            self.id += 1;
            return Ok(id);
        }

        Ok(id)
    }
    */

    pub fn close(&mut self, id: Id) -> Result<(), Box<dyn Error>> {
        self.map.remove(&id).ok_or("Invalid id")?;
        Ok(())
    }


    pub fn write_mem(&self, id: Id, offset: usize, data: usize) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem(offset, data);
        }
        Ok(())
    }

    pub fn write_mem_u8(&self, id: Id, offset: usize, data: u8) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u8(offset, data);
        }
        Ok(())
    }

    pub fn write_reg_u8(&self, id: Id, reg: usize, data: u8) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u8(reg * unit, data);
        }
        Ok(())
    }

    pub fn write_mem_u16(
        &mut self,
        id: Id,
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
        id: Id,
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
        id: Id,
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

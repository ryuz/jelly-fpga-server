#![allow(dead_code)]

use jelly_mem_access::mem_accessor::MemAccess;
use jelly_mem_access::MmapAccessor;
use jelly_mem_access::UioAccessor;
use jelly_mem_access::UdmabufAccessor;
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;

pub type Id = u32;

#[derive(Debug)]
enum AccessorEnum {
    MmapAccessor(MmapAccessor<u8>),
    UioAccessor(UioAccessor<u8>),
    UdmabufAccessor(UdmabufAccessor<u8>),
}

#[derive(Debug)]
pub struct Accessor {
    id: Id,
    map: HashMap<Id, (AccessorEnum, usize)>,
}

impl Default for Accessor {
    fn default() -> Self {
        Self {
            id: 1,
            map: HashMap::new(),
        }
    }
}

impl Accessor {
    pub fn new() -> Self {
        Accessor {
            id: 1,
            map: HashMap::new(),
        }
    }

    fn add_accessor(&mut self, accessor: AccessorEnum, unit: usize) -> Id{
        let unit = if unit == 0 { std::mem::size_of::<usize>() } else { unit };
        let id = self.id;
        self.map.insert(id, (accessor, unit));
        self.id += 1;
        id
    }

    pub fn open_mmap(&mut self, path: &str, offset: usize, size: usize, unit: usize) -> Result<Id, Box<dyn Error>> {
        let accessor = MmapAccessor::<u8>::new(path, offset, size)?;
        let id = self.add_accessor(AccessorEnum::MmapAccessor(accessor), unit);
        Ok(id)
    }

    pub fn open_uio(&mut self, name: &str, unit: usize) -> Result<Id, Box<dyn Error>> {
        let accessor = UioAccessor::<u8>::new_with_name(name)?;
        let id = self.add_accessor(AccessorEnum::UioAccessor(accessor), unit);
        Ok(id)
    }

    pub fn open_udmabuf(
        &mut self,
        name: &str,
        cache_enable: bool,
        unit: usize,
    ) -> Result<Id, Box<dyn Error>> {
        let accessor = UdmabufAccessor::<u8>::new(name, cache_enable)?;
        let id = self.add_accessor(AccessorEnum::UdmabufAccessor(accessor), unit);
        Ok(id)
    }

    fn accessor(&self, id: Id) -> Result<(&dyn MemAccess, usize), Box<dyn Error>> {
        let (accessor, unit) = self.map.get(&id).ok_or("Invalid id")?;
        match accessor {
            AccessorEnum::MmapAccessor(acc) => return Ok((acc, *unit)),
            AccessorEnum::UioAccessor(acc) => return Ok((acc, *unit)),
            AccessorEnum::UdmabufAccessor(acc) => return Ok((acc, *unit)),
            //        _ => return Err("Invalid accessor".into()),
        }
    }

    pub fn subclone(
        &mut self,
        id: Id,
        offset: usize,
        size: usize,
        unit: usize,
    ) -> Result<Id, Box<dyn Error>> {
        let (accessor, unit_org) = self.map.get(&id).ok_or("Invalid id")?;
        let unit = if unit == 0 { *unit_org } else { unit };
        let accessor: AccessorEnum = match accessor {
            AccessorEnum::MmapAccessor(acc) => {
                let acc = acc.subclone8(offset, size);
                AccessorEnum::MmapAccessor(acc)
            }
            AccessorEnum::UioAccessor(acc) => {
                let acc = acc.subclone8(offset, size);
                AccessorEnum::UioAccessor(acc)
            }
            AccessorEnum::UdmabufAccessor(acc) => {
                let acc = acc.subclone8(offset, size);
                AccessorEnum::UdmabufAccessor(acc)
            }
        };
        let id = self.id;
        self.map.insert(id, (accessor, unit));
        self.id += 1;
        Ok(id)
    }

    pub fn close(&mut self, id: Id) -> Result<(), Box<dyn Error>> {
        self.map.remove(&id).ok_or("Invalid id")?;
        Ok(())
    }

    pub fn close_all(&mut self) {
        self.map.clear();
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

    pub fn read_mem_u8(&self, id: Id, offset: usize) -> Result<u8, Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u8(offset) })
    }

    pub fn read_reg_u8(&self, id: Id, reg: usize) -> Result<u8, Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u8(reg * unit) })
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

#![allow(dead_code)]

use jelly_mem_access::mem_accessor::MemAccess;
use jelly_mem_access::MmapAccessor;
use jelly_mem_access::UdmabufAccessor;
use jelly_mem_access::UioAccessor;
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

    fn add_accessor(&mut self, accessor: AccessorEnum, unit: usize) -> Id {
        let unit = if unit == 0 {
            std::mem::size_of::<usize>()
        } else {
            unit
        };
        let id = self.id;
        self.map.insert(id, (accessor, unit));
        self.id += 1;
        id
    }

    pub fn open_mmap(
        &mut self,
        path: &str,
        offset: usize,
        size: usize,
        unit: usize,
    ) -> Result<Id, Box<dyn Error>> {
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

    pub unsafe fn write_mem_u(
        &mut self,
        id: Id,
        offset: usize,
        data: u64,
        size: usize,
    ) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        match size {
            0 => accessor.write_mem_usize(offset, data as usize),
            1 => accessor.write_mem_u8(offset, data as u8),
            2 => accessor.write_mem_u16(offset, data as u16),
            4 => accessor.write_mem_u32(offset, data as u32),
            8 => accessor.write_mem_u64(offset, data),
            _ => return Err("Invalid size".into()),
        };
        Ok(())
    }

    pub unsafe fn write_mem_i(
        &mut self,
        id: Id,
        offset: usize,
        data: i64,
        size: usize,
    ) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        match size {
            0 => accessor.write_mem_isize(offset, data as isize),
            1 => accessor.write_mem_i8(offset, data as i8),
            2 => accessor.write_mem_i16(offset, data as i16),
            4 => accessor.write_mem_i32(offset, data as i32),
            8 => accessor.write_mem_i64(offset, data),
            _ => return Err("Invalid size".into()),
        };
        Ok(())
    }

    pub unsafe fn read_mem_u(
        &mut self,
        id: Id,
        offset: usize,
        size: usize,
    ) -> Result<u64, Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        let data = match size {
            0 => accessor.read_mem_usize(offset) as u64,
            1 => accessor.read_mem_u8(offset) as u64,
            2 => accessor.read_mem_u16(offset) as u64,
            4 => accessor.read_mem_u32(offset) as u64,
            8 => accessor.read_mem_u64(offset),
            _ => return Err("Invalid size".into()),
        };
        Ok(data)
    }

    pub unsafe fn read_mem_i(
        &mut self,
        id: Id,
        offset: usize,
        size: usize,
    ) -> Result<i64, Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        let data = match size {
            0 => accessor.read_mem_isize(offset) as i64,
            1 => accessor.read_mem_i8(offset) as i64,
            2 => accessor.read_mem_i16(offset) as i64,
            4 => accessor.read_mem_i32(offset) as i64,
            8 => accessor.read_mem_i64(offset),
            _ => return Err("Invalid size".into()),
        };
        Ok(data)
    }

    pub unsafe fn write_reg_u(
        &mut self,
        id: Id,
        reg: usize,
        data: u64,
        size: usize,
    ) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        match size {
            0 => accessor.write_mem_usize(reg * unit, data as usize),
            1 => accessor.write_mem_u8(reg * unit, data as u8),
            2 => accessor.write_mem_u16(reg * unit, data as u16),
            4 => accessor.write_mem_u32(reg * unit, data as u32),
            8 => accessor.write_mem_u64(reg * unit, data as u64),
            _ => return Err("Invalid size".into()),
        };
        Ok(())
    }

    pub unsafe fn write_reg_i(
        &mut self,
        id: Id,
        reg: usize,
        data: i64,
        size: usize,
    ) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        match size {
            0 => accessor.write_mem_isize(reg * unit, data as isize),
            1 => accessor.write_mem_i8(reg * unit, data as i8),
            2 => accessor.write_mem_i16(reg * unit, data as i16),
            4 => accessor.write_mem_i32(reg * unit, data as i32),
            8 => accessor.write_mem_i64(reg * unit, data),
            _ => return Err("Invalid size".into()),
        };
        Ok(())
    }

    pub unsafe fn read_reg_u(
        &mut self,
        id: Id,
        reg: usize,
        size: usize,
    ) -> Result<u64, Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        let data = match size {
            0 => accessor.read_mem_usize(reg * unit) as u64,
            1 => accessor.read_mem_u8(reg * unit) as u64,
            2 => accessor.read_mem_u16(reg * unit) as u64,
            4 => accessor.read_mem_u32(reg * unit) as u64,
            8 => accessor.read_mem_u64(reg * unit),
            _ => return Err("Invalid size".into()),
        };
        Ok(data)
    }

    pub unsafe fn read_reg_i(
        &mut self,
        id: Id,
        reg: usize,
        size: usize,
    ) -> Result<i64, Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        let data = match size {
            0 => accessor.read_mem_isize(reg * unit) as i64,
            1 => accessor.read_mem_i8(reg * unit) as i64,
            2 => accessor.read_mem_i16(reg * unit) as i64,
            4 => accessor.read_mem_i32(reg * unit) as i64,
            8 => accessor.read_mem_i64(reg * unit),
            _ => return Err("Invalid size".into()),
        };
        Ok(data)
    }

    /*
    pub fn write_mem_usize(&mut self, id: Id, offset: usize, data: usize) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem(offset, data);
        }
        Ok(())
    }

    pub fn write_mem_u8(&mut self, id: Id, offset: usize, data: u8) -> Result<(), Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u8(offset, data);
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

    pub fn read_mem_u8(&mut self, id: Id, offset: usize) -> Result<u8, Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u8(offset) })
    }

    pub fn read_mem_u16(&mut self, id: Id, offset: usize) -> Result<u16, Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u16(offset) })
    }

    pub fn read_mem_u32(&mut self, id: Id, offset: usize) -> Result<u32, Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u32(offset) })
    }

    pub fn read_mem_u64(&mut self, id: Id, offset: usize) -> Result<u64, Box<dyn Error>> {
        let (accessor, _) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u64(offset) })
    }


    pub fn write_reg(&mut self, id: Id, reg: usize, data: usize) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        unsafe {
            accessor.write_mem(reg * unit, data);
        }
        Ok(())
    }

    pub fn write_reg_u8(&mut self, id: Id, reg: usize, data: u8) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u8(reg * unit, data);
        }
        Ok(())
    }

    pub fn write_reg_u16(&mut self, id: Id, reg: usize, data: u16) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u16(reg * unit, data);
        }
        Ok(())
    }

    pub fn write_reg_u32(&mut self, id: Id, reg: usize, data: u32) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u32(reg * unit, data);
        }
        Ok(())
    }

    pub fn write_reg_u64(&mut self, id: Id, reg: usize, data: u64) -> Result<(), Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        unsafe {
            accessor.write_mem_u64(reg * unit, data);
        }
        Ok(())
    }

    pub fn read_reg(&mut self, id: Id, reg: usize) -> Result<usize, Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem(reg * unit) })
    }

    pub fn read_reg_u8(&mut self, id: Id, reg: usize) -> Result<u8, Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u8(reg * unit) })
    }

    pub fn read_reg_u16(&mut self, id: Id, reg: usize) -> Result<u16, Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u16(reg * unit) })
    }

    pub fn read_reg_u32(&mut self, id: Id, reg: usize) -> Result<u32, Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u32(reg * unit) })
    }

    pub fn read_reg_u64(&mut self, id: Id, reg: usize) -> Result<u64, Box<dyn Error>> {
        let (accessor, unit) = self.accessor(id)?;
        Ok(unsafe { accessor.read_mem_u64(reg * unit) })
    }
    */
}

use std::sync::Arc;
use fpga_control::fpga_control_server::*;
use fpga_control::*;
//use tokio::fs::File;
//use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tokio::sync::RwLock;

use jelly_fpgautil as fpgautil;
use jelly_uidmng as uidmng;

pub mod fpga_control {
    tonic::include_proto!("fpga_control");
}

mod accessor;
use accessor::Accessor;

#[derive(Debug, Default)]
pub struct FpgaControlService {
    verbose: i32,
    accessor: Arc<RwLock<Accessor>>,
}


#[tonic::async_trait]
impl FpgaControl for FpgaControlService {
    async fn reset(
        &self,
        _request: Request<ResetRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        if self.verbose >= 1 {
            println!("reset");
        }
        let mut accessor = self.accessor.write().await;
        accessor.close_all();
        Ok(Response::new(BoolResponse { result: true }))
    }

    async fn load(&self, request: Request<LoadRequest>) -> Result<Response<LoadResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("load : {}", req.name);
        }
        let result = fpgautil::load(&req.name);
        if let Ok(slot) = result {
            Ok(Response::new(LoadResponse {
                result: true,
                slot: slot,
            }))
        } else {
            Ok(Response::new(LoadResponse {
                result: false,
                slot: -1,
            }))
        }
    }

    async fn unload(
        &self,
        request: Request<UnloadRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("unload : {}", req.slot);
        }
        let result = fpgautil::unload(req.slot);
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn upload_firmware(
        &self,
        request: Request<Streaming<UploadFirmwareRequest>>,
    ) -> Result<Response<BoolResponse>, Status> {
        let mut stream = request.into_inner();

        let mut first = true;
        while let Some(msg) = stream.next().await {
            let msg = msg?;
            let name = format!("/lib/firmware/{}", msg.name);
            if let Err(e) = if first {
                uidmng::write_sudo(&name, &msg.data)
            } else {
                uidmng::append_sudo(&name, &msg.data)
            } {
                println!("Error:{}", e);
                return Ok(Response::new(BoolResponse { result: false }));
            }
            first = false;
        }

        Ok(Response::new(BoolResponse { result: true }))
    }

    async fn load_bitstream(
        &self,
        request: Request<LoadBitstreamRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("load_bitstream : {}", req.name);
        }
        let result = fpgautil::load_bitstream_from_firmware(&req.name);
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn load_dtbo(
        &self,
        request: Request<LoadDtboRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("load_dtbo : {}", req.name);
        }
        let result = fpgautil::load_dtbo_from_firmware(&req.name);
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn dts_to_dtb(
        &self,
        request: Request<DtsToDtbRequest>,
    ) -> Result<Response<DtsToDtbResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("dts_to_dtb");
        }
        let result = fpgautil::dtc_with_str(&req.dts);
        match result {
            Ok(dtb) => Ok(Response::new(DtsToDtbResponse {
                result: true,
                dtb: dtb,
            })),
            Err(e) => {
                println!("Error:{}", e);
                Ok(Response::new(DtsToDtbResponse {
                    result: false,
                    dtb: [].to_vec(),
                }))
            }
        }
    }

    async fn open_mmap(
        &self,
        request: Request<OpenMmapRequest>,
    ) -> Result<Response<OpenResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("open_mmap:{}", req.path);
        }
        let mut accessor = self.accessor.write().await;
        let result = accessor.open_mmap(&req.path, req.offset as usize, req.size as usize, req.unit as usize);
        match result {
            Ok(id) => Ok(Response::new(OpenResponse {
                result: true,
                id: id,
            })),
            Err(_) => Ok(Response::new(OpenResponse {
                result: false,
                id: 0,
            })),
        }
    }

    async fn open_uio(
        &self,
        request: Request<OpenUioRequest>,
    ) -> Result<Response<OpenResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("open_uio:{}", req.name);
        }
        let mut accessor = self.accessor.write().await;
        let result = accessor.open_uio(&req.name, req.unit as usize);
        match result {
            Ok(id) => Ok(Response::new(OpenResponse {
                result: true,
                id: id,
            })),
            Err(_) => Ok(Response::new(OpenResponse {
                result: false,
                id: 0,
            })),
        }
    }

    async fn close(
        &self,
        request: Request<CloseRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("close:{}", req.id);
        }
        let mut accessor = self.accessor.write().await;
        let result = accessor.close(req.id as accessor::Id);
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn write_mem_u(
        &self,
        request: Request<WriteMemURequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("write_mem:id={} offset={} data={} size={}", req.id, req.offset, req.data, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.write_mem_u(req.id as accessor::Id, req.offset as usize, req.data, req.size as usize) };
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn write_mem_i(
        &self,
        request: Request<WriteMemIRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("write_mem:id={} offset={} data={} size={}", req.id, req.offset, req.data, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.write_mem_i(req.id as accessor::Id, req.offset as usize, req.data, req.size as usize) };
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn read_mem_u(
        &self,
        request: Request<ReadMemRequest>,
    ) -> Result<Response<ReadUResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_mem_u:id={} offset={} size={}", req.id, req.offset, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.read_mem_u(req.id as accessor::Id, req.offset as usize, req.size as usize) };
        match result {
            Ok(data) => Ok(Response::new(ReadUResponse { result: true, data: data })),
            Err(_) => Ok(Response::new(ReadUResponse { result: false, data: 0 })),
        }
    }

    async fn read_mem_i(
        &self,
        request: Request<ReadMemRequest>,
    ) -> Result<Response<ReadIResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_mem_u:id={} offset={} size={}", req.id, req.offset, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.read_mem_i(req.id as accessor::Id, req.offset as usize, req.size as usize) };
        match result {
            Ok(data) => Ok(Response::new(ReadIResponse { result: true, data: data })),
            Err(_) => Ok(Response::new(ReadIResponse { result: false, data: 0 })),
        }
    }

    async fn write_reg_u(
        &self,
        request: Request<WriteRegURequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("write_mem:id={} offset={} data={} size={}", req.id, req.reg, req.data, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.write_reg_u(req.id as accessor::Id, req.reg as usize, req.data, req.size as usize) };
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn write_reg_i(
        &self,
        request: Request<WriteRegIRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("write_mem:id={} offset={} data={} size={}", req.id, req.reg, req.data, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.write_reg_i(req.id as accessor::Id, req.reg as usize, req.data, req.size as usize) };
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn read_reg_u(
        &self,
        request: Request<ReadRegRequest>,
    ) -> Result<Response<ReadUResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_mem_u:id={} offset={} size={}", req.id, req.reg, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.read_reg_u(req.id as accessor::Id, req.reg as usize, req.size as usize) };
        match result {
            Ok(data) => Ok(Response::new(ReadUResponse { result: true, data: data })),
            Err(_) => Ok(Response::new(ReadUResponse { result: false, data: 0 })),
        }
    }

    async fn read_reg_i(
        &self,
        request: Request<ReadRegRequest>,
    ) -> Result<Response<ReadIResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_reg_i:id={} offset={} size={}", req.id, req.reg, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.read_reg_i(req.id as accessor::Id, req.reg as usize, req.size as usize) };
        match result {
            Ok(data) => Ok(Response::new(ReadIResponse { result: true, data: data })),
            Err(_) => Ok(Response::new(ReadIResponse { result: false, data: 0 })),
        }
    }

    /*
    async fn write_reg_u(
        &self,
        request: Request<WriteURequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("write_mem:id={} addr={} data={} size={}", req.id, req.addr, req.data, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = match req.size {
            0 => accessor.write_reg(req.id as accessor::Id, req.addr as usize, req.data as usize),
            1 => accessor.write_reg_u8(req.id as accessor::Id, req.addr as usize, req.data as u8),
            2 => accessor.write_reg_u16(req.id as accessor::Id, req.addr as usize, req.data as u16),
            4 => accessor.write_reg_u32(req.id as accessor::Id, req.addr as usize, req.data as u32),
            8 => accessor.write_reg_u64(req.id as accessor::Id, req.addr as usize, req.data as u64),
            _ => return Ok(Response::new(BoolResponse { result: false })),
        };
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn write_mem_s(
        &self,
        request: Request<WriteSRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("write_mem:id={} addr={} data={} size={}", req.id, req.addr, req.data, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = match req.size {
            0 => accessor.write_mem(req.id as accessor::Id, req.addr as usize, req.data as usize),
            1 => accessor.write_mem_u8(req.id as accessor::Id, req.addr as usize, req.data as u8),
            2 => accessor.write_mem_u16(req.id as accessor::Id, req.addr as usize, req.data as u16),
            4 => accessor.write_mem_u32(req.id as accessor::Id, req.addr as usize, req.data as u32),
            8 => accessor.write_mem_u64(req.id as accessor::Id, req.addr as usize, req.data as u64),
            _ => return Ok(Response::new(BoolResponse { result: false })),
        };
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn write_reg_s(
        &self,
        request: Request<WriteSRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("write_mem:id={} addr={} data={} size={}", req.id, req.addr, req.data, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = match req.size {
            0 => accessor.write_reg(req.id as accessor::Id, req.addr as usize, req.data as usize),
            1 => accessor.write_reg_u8(req.id as accessor::Id, req.addr as usize, req.data as u8),
            2 => accessor.write_reg_u16(req.id as accessor::Id, req.addr as usize, req.data as u16),
            4 => accessor.write_reg_u32(req.id as accessor::Id, req.addr as usize, req.data as u32),
            8 => accessor.write_reg_u64(req.id as accessor::Id, req.addr as usize, req.data as u64),
            _ => return Ok(Response::new(BoolResponse { result: false })),
        };
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn read_mem_u(
        &self,
        request: Request<ReadRequest>,
    ) -> Result<Response<ReadUResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_mem:id={} addr={} size={}", req.id, req.addr, req.size);
        }
        let mut accessor = self.accessor.write().await;
        let result = match req.size {
            0 => accessor.read_mem    (req.id as accessor::Id, req.addr as usize) as u64,
            1 => accessor.read_mem_u8 (req.id as accessor::Id, req.addr as usize) as u64,
            2 => accessor.read_mem_u16(req.id as accessor::Id, req.addr as usize) as u64,
            4 => accessor.read_mem_u32(req.id as accessor::Id, req.addr as usize) as u64,
            8 => accessor.read_mem_u64(req.id as accessor::Id, req.addr as usize) as u64,
            _ => return Ok(Response::new(ReadUResponse { result: false, data : 0 })),
        };
        match result {
            Ok(data) => Ok(Response::new(ReadUResponse { result: true, data: data })),
            Err(_) => Ok(Response::new(ReadUResponse { result: false, data: 0 })),
        }
    }
    */
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fpgautil::set_allow_sudo(true);

    let address = "0.0.0.0:50051".parse().unwrap();
    let mut fpga_contro_service = FpgaControlService::default();
    fpga_contro_service.verbose = 0;

    Server::builder()
        .add_service(FpgaControlServer::new(fpga_contro_service))
        .serve(address)
        .await?;

    Ok(())
}

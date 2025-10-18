use jelly_fpgautil as fpgautil;
use jelly_uidmng as uidmng;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tonic::{transport::Server, Request, Response, Status, Streaming};

pub mod jelly_fpga_control {
    tonic::include_proto!("jelly_fpga_control");
}

use jelly_fpga_control::jelly_fpga_control_server::*;
use jelly_fpga_control::*;

mod accessor;
use accessor::Accessor;

#[derive(Debug, Default)]
struct JellyFpgaControlService {
    verbose: i32,
    accessor: Arc<RwLock<Accessor>>,
}

impl JellyFpgaControlService {
    pub fn new(verbose: i32) -> Self {
        JellyFpgaControlService {
            verbose,
            accessor: Arc::new(RwLock::new(Accessor::new())),
        }
    }
}

#[tonic::async_trait]
impl JellyFpgaControl for JellyFpgaControlService {
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
            println!("load: name={}", req.name);
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
            println!("unload: slot={}", req.slot);
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
        if self.verbose >= 1 {
            println!("upload_firmware");
        }

        let mut stream = request.into_inner();

        let mut first = true;
        while let Some(msg) = stream.next().await {
            if self.verbose >= 2 {
                print!(".");
            }
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

        if self.verbose >= 2 {
            println!("done");
        }

        Ok(Response::new(BoolResponse { result: true }))
    }

    async fn remove_firmware(
        &self,
        request: Request<RemoveFirmwareRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("remove_firmware: name={}", req.name);
        }
        let result = fpgautil::remove_firmware(&req.name);
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn load_bitstream(
        &self,
        request: Request<LoadBitstreamRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("load_bitstream: name={}", req.name);
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
            println!("load_dtbo: name={}", req.name);
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

    async fn bitstream_to_bin(
        &self,
        request: Request<BitstreamToBinRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "bitstream_to_bin: bitstream_name={} bin_name={} arch={}",
                req.bitstream_name, req.bin_name, req.arch
            );
        }
        let bit_path = format!("/lib/firmware/{}", req.bitstream_name);
        let bin_path = format!("/lib/firmware/{}", req.bin_name);
        let result = fpgautil::xlnx_bitstream_to_bin(&bit_path, &bin_path, &req.arch);
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn open_mmap(
        &self,
        request: Request<OpenMmapRequest>,
    ) -> Result<Response<OpenResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("open_mmap: path={}", req.path);
        }
        let mut accessor = self.accessor.write().await;
        let result = accessor.open_mmap(
            &req.path,
            req.offset as usize,
            req.size as usize,
            req.unit as usize,
        );
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
            println!("open_uio: name={}", req.name);
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

    async fn open_udmabuf(
        &self,
        request: Request<OpenUdmabufRequest>,
    ) -> Result<Response<OpenResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("open_udmabuf: name={}", req.name);
        }
        let mut accessor = self.accessor.write().await;
        let result = accessor.open_udmabuf(&req.name, req.cache_enable, req.unit as usize);
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

    async fn subclone(
        &self,
        request: Request<SubcloneRequest>,
    ) -> Result<Response<SubcloneResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("subclone: id={} offset={} size={} unit={}", req.id, req.offset, req.size, req.unit);
        }
        let mut accessor = self.accessor.write().await;
        let result = accessor.subclone(req.id as accessor::Id, req.offset as usize, req.size as usize, req.unit as usize);
        match result {
            Ok(id) => Ok(Response::new(SubcloneResponse {
                result: true,
                id: id,
            })),
            Err(_) => Ok(Response::new(SubcloneResponse {
                result: false,
                id: 0,
            })),
        }
    }

    async fn get_addr(
        &self,
        request: Request<GetAddrRequest>,
    ) -> Result<Response<GetAddrResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("get_addr: id={}", req.id);
        }
        let accessor = self.accessor.read().await;
        let result = accessor.addr(req.id as accessor::Id);
        match result {
            Ok(addr) => Ok(Response::new(GetAddrResponse {
                result: true,
                addr: addr as u64,
            })),
            Err(_) => Ok(Response::new(GetAddrResponse {
                result: false,
                addr: 0,
            })),
        }
    }

    async fn get_size(
        &self,
        request: Request<GetSizeRequest>,
    ) -> Result<Response<GetSizeResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("get_size: id={}", req.id);
        }
        let accessor = self.accessor.read().await;
        let result = accessor.size(req.id as accessor::Id);
        match result {
            Ok(size) => Ok(Response::new(GetSizeResponse {
                result: true,
                size: size as u64,
            })),
            Err(_) => Ok(Response::new(GetSizeResponse {
                result: false,
                size: 0,
            })),
        }
    }

    async fn get_phys_addr(
        &self,
        request: Request<GetPhysAddrRequest>,
    ) -> Result<Response<GetPhysAddrResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("get_phys_addr: id={}", req.id);
        }
        let accessor = self.accessor.read().await;
        let result = accessor.phys_addr(req.id as accessor::Id);
        match result {
            Ok(phys_addr) => Ok(Response::new(GetPhysAddrResponse {
                result: true,
                phys_addr: phys_addr as u64,
            })),
            Err(_) => Ok(Response::new(GetPhysAddrResponse {
                result: false,
                phys_addr: 0,
            })),
        }
    }

    async fn close(
        &self,
        request: Request<CloseRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("close: id={}", req.id);
        }
        let mut accessor = self.accessor.write().await;
        let result = accessor.close(req.id as accessor::Id);
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn write_mem_u(
        &self,
        request: Request<WriteMemURequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "write_mem_u: id={} offset={} data={} size={}",
                req.id, req.offset, req.data, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.write_mem_u(
                req.id as accessor::Id,
                req.offset as usize,
                req.data,
                req.size as usize,
            )
        };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn write_mem_i(
        &self,
        request: Request<WriteMemIRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "write_mem_i: id={} offset={} data={} size={}",
                req.id, req.offset, req.data, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.write_mem_i(
                req.id as accessor::Id,
                req.offset as usize,
                req.data,
                req.size as usize,
            )
        };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn read_mem_u(
        &self,
        request: Request<ReadMemRequest>,
    ) -> Result<Response<ReadUResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "read_mem_u: id={} offset={} size={}",
                req.id, req.offset, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.read_mem_u(
                req.id as accessor::Id,
                req.offset as usize,
                req.size as usize,
            )
        };
        match result {
            Ok(data) => Ok(Response::new(ReadUResponse {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(ReadUResponse {
                result: false,
                data: 0,
            })),
        }
    }

    async fn read_mem_i(
        &self,
        request: Request<ReadMemRequest>,
    ) -> Result<Response<ReadIResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "read_mem_i: id={} offset={} size={}",
                req.id, req.offset, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.read_mem_i(
                req.id as accessor::Id,
                req.offset as usize,
                req.size as usize,
            )
        };
        match result {
            Ok(data) => Ok(Response::new(ReadIResponse {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(ReadIResponse {
                result: false,
                data: 0,
            })),
        }
    }

    async fn write_reg_u(
        &self,
        request: Request<WriteRegURequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "write_reg_u: id={} reg={} data={} size={}",
                req.id, req.reg, req.data, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.write_reg_u(
                req.id as accessor::Id,
                req.reg as usize,
                req.data,
                req.size as usize,
            )
        };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn write_reg_i(
        &self,
        request: Request<WriteRegIRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "write_reg_i: id={} reg={} data={} size={}",
                req.id, req.reg, req.data, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.write_reg_i(
                req.id as accessor::Id,
                req.reg as usize,
                req.data,
                req.size as usize,
            )
        };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn read_reg_u(
        &self,
        request: Request<ReadRegRequest>,
    ) -> Result<Response<ReadUResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "read_reg_u: id={} reg={} size={}",
                req.id, req.reg, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.read_reg_u(req.id as accessor::Id, req.reg as usize, req.size as usize)
        };
        match result {
            Ok(data) => Ok(Response::new(ReadUResponse {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(ReadUResponse {
                result: false,
                data: 0,
            })),
        }
    }

    async fn read_reg_i(
        &self,
        request: Request<ReadRegRequest>,
    ) -> Result<Response<ReadIResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "read_reg_i: id={} reg={} size={}",
                req.id, req.reg, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.read_reg_i(req.id as accessor::Id, req.reg as usize, req.size as usize)
        };
        match result {
            Ok(data) => Ok(Response::new(ReadIResponse {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(ReadIResponse {
                result: false,
                data: 0,
            })),
        }
    }

    async fn write_mem_f32(
        &self,
        request: Request<WriteMemF32Request>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "write_mem_f32: id={} offset={} data={}",
                req.id, req.offset, req.data
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.write_mem_f32(req.id as accessor::Id, req.offset as usize, req.data)
        };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn write_mem_f64(
        &self,
        request: Request<WriteMemF64Request>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "write_mem_f64: id={} offset={} data={}",
                req.id, req.offset, req.data
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.write_mem_f64(req.id as accessor::Id, req.offset as usize, req.data)
        };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn read_mem_f32(
        &self,
        request: Request<ReadMemRequest>,
    ) -> Result<Response<ReadF32Response>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_mem_f32: id={} offset={}", req.id, req.offset);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.read_mem_f32(req.id as accessor::Id, req.offset as usize) };
        match result {
            Ok(data) => Ok(Response::new(ReadF32Response {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(ReadF32Response {
                result: false,
                data: 0.0,
            })),
        }
    }

    async fn read_mem_f64(
        &self,
        request: Request<ReadMemRequest>,
    ) -> Result<Response<ReadF64Response>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_mem_f64: id={} offset={}", req.id, req.offset);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.read_mem_f64(req.id as accessor::Id, req.offset as usize) };
        match result {
            Ok(data) => Ok(Response::new(ReadF64Response {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(ReadF64Response {
                result: false,
                data: 0.0,
            })),
        }
    }

    async fn write_reg_f32(
        &self,
        request: Request<WriteRegF32Request>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "write_reg_f32: id={} reg={} data={}",
                req.id, req.reg, req.data
            );
        }
        let mut accessor = self.accessor.write().await;
        let result =
            unsafe { accessor.write_reg_f32(req.id as accessor::Id, req.reg as usize, req.data) };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn write_reg_f64(
        &self,
        request: Request<WriteRegF64Request>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "write_reg_f64: id={} reg={} data={}",
                req.id, req.reg, req.data
            );
        }
        let mut accessor = self.accessor.write().await;
        let result =
            unsafe { accessor.write_reg_f64(req.id as accessor::Id, req.reg as usize, req.data) };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn read_reg_f32(
        &self,
        request: Request<ReadRegRequest>,
    ) -> Result<Response<ReadF32Response>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_reg_f32: id={} reg={}", req.id, req.reg);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.read_reg_f32(req.id as accessor::Id, req.reg as usize) };
        match result {
            Ok(data) => Ok(Response::new(ReadF32Response {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(ReadF32Response {
                result: false,
                data: 0.0,
            })),
        }
    }

    async fn read_reg_f64(
        &self,
        request: Request<ReadRegRequest>,
    ) -> Result<Response<ReadF64Response>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!("read_reg_f64: id={} reg={}", req.id, req.reg);
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe { accessor.read_reg_f64(req.id as accessor::Id, req.reg as usize) };
        match result {
            Ok(data) => Ok(Response::new(ReadF64Response {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(ReadF64Response {
                result: false,
                data: 0.0,
            })),
        }
    }

    async fn mem_copy_to(
        &self,
        request: Request<MemCopyToRequest>,
    ) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "mem_copy_to: id={} offset={}, len={}",
                req.id,
                req.offset,
                req.data.len()
            );
        }
        let mut accessor = self.accessor.write().await;
        let result =
            unsafe { accessor.mem_copy_to(req.id as accessor::Id, req.offset as usize, &req.data) };
        Ok(Response::new(BoolResponse {
            result: result.is_ok(),
        }))
    }

    async fn mem_copy_from(
        &self,
        request: Request<MemCopyFromRequest>,
    ) -> Result<Response<MemCopyFromResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 {
            println!(
                "mem_copy_from: id={} offset={} size={}",
                req.id, req.offset, req.size
            );
        }
        let mut accessor = self.accessor.write().await;
        let result = unsafe {
            accessor.mem_copy_from(
                req.id as accessor::Id,
                req.offset as usize,
                req.size as usize,
            )
        };
        match result {
            Ok(data) => Ok(Response::new(MemCopyFromResponse {
                result: true,
                data: data,
            })),
            Err(_) => Ok(Response::new(MemCopyFromResponse {
                result: false,
                data: [].to_vec(),
            })),
        }
    }
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Verbose level
    #[arg(short, long, default_value_t = 0)]
    verbose: i32,
    /// Allow external connections
    #[arg(long)]
    external: bool,
    /// Port number to listen on
    #[arg(short, long, default_value_t = 8051)]
    port: u16,
    #[arg(long)]
    allow_sudo: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.allow_sudo {
        fpgautil::set_allow_sudo(true);
    }

    let fpga_control_service = JellyFpgaControlService::new(args.verbose);

    let address = if args.external {
        format!("0.0.0.0:{}", args.port)
    } else {
        format!("127.0.0.1:{}", args.port)
    }
    .parse()
    .unwrap();

    if args.verbose >= 1 {
        println!("jelly-fpga-server start");
    }

    Server::builder()
        .add_service(JellyFpgaControlServer::new(fpga_control_service))
        .serve(address)
        .await?;

    if args.verbose >= 1 {
        println!("jelly-fpga-server stop");
    }

    Ok(())
}

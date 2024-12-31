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



use fpga_control::*;
use fpga_control::fpga_control_server::*;
use tonic::{transport::Server, Request, Response, Status};

use jelly_fpgautil as fpgautil;


pub mod fpga_control {
    tonic::include_proto!("fpga_control");
}

#[derive(Debug, Default)]
pub struct FpgaControlService {
    verbose : i32,
}

//mod fpga;
//use fpga::CameraManager;

struct FpgaManager {
}

impl FpgaManager {
    fn new() -> Self {
        FpgaManager {}
    }
}


use once_cell::sync::Lazy;
use std::sync::Mutex;
static FPGA_CTL: Lazy<Mutex<FpgaManager>> = Lazy::new(|| Mutex::new(FpgaManager::new()));


#[tonic::async_trait]
impl FpgaControl for FpgaControlService {

    async fn reset(&self, request: Request<ResetRequest>) -> Result<Response<BoolResponse>, Status> {
//      let req = request.into_inner();
        if self.verbose >= 1 { println!("reset"); }
        Ok(Response::new(BoolResponse { result: true }))
    }

    async fn load(&self, request: Request<LoadRequest>) -> Result<Response<LoadResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 { println!("load : {}", req.name); }
        let result = fpgautil::load(&req.name);
        if let Ok(slot) = result {
            Ok(Response::new(LoadResponse { result: true, slot: slot }))
        } 
        else {
            Ok(Response::new(LoadResponse { result: false, slot: -1 }))
        }
    }

    async fn unload(&self, request: Request<UnloadRequest>) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 { println!("unload : {}", req.slot); }
        let result = fpgautil::unload(req.slot);
        Ok(Response::new(BoolResponse { result: result.is_ok() }))
    }

    async fn open_uio(&self, request: Request<OpenUioRequest>) -> Result<Response<OpenResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 { println!("open_uio:{}", req.name); }
//        let result = match FPGA_CTL.lock().unwrap().open() {Ok(_) => true, Err(_) => false};
        let id = 0;
        Ok(Response::new(OpenResponse { result: true, id: id }))
    }

    async fn close(&self, request: Request<CloseRequest>) -> Result<Response<BoolResponse>, Status> {
        let req = request.into_inner();
        if self.verbose >= 1 { println!("close:{}", req.id); }
//      FPGA_CTL.lock().unwrap().close();
        Ok(Response::new(BoolResponse { result: true }))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    fpgautil::set_allow_sudo(true);
//    FPGA_CTL.lock().unwrap().open()?;

    let address = "0.0.0.0:50051".parse().unwrap();
    let mut fpga_contro_service = FpgaControlService::default();
    fpga_contro_service.verbose = 0;

    Server::builder()
        .add_service(FpgaControlServer::new(fpga_contro_service))
        .serve(address)
        .await?;
//   FPGA_CTL.lock().unwrap().close();

    Ok(())
}

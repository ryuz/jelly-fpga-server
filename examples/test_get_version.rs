use std::env;

pub mod jelly_fpga_control {
    tonic::include_proto!("jelly_fpga_control");
}

use jelly_fpga_control::jelly_fpga_control_client::JellyFpgaControlClient;
use jelly_fpga_control::Empty;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // コマンドライン引数でIPアドレスを取得（省略可能）
    let args: Vec<String> = env::args().collect();
    let target = if args.len() >= 2 {
        &args[1]
    } else {
        "127.0.0.1:8051" // デフォルト
    };
    
    println!("Connecting to: {}", target);
    
    // サーバーに接続
    let target_url = if target.starts_with("http://") {
        target.to_string()
    } else {
        format!("http://{}", target)
    };

    let mut client = JellyFpgaControlClient::connect(target_url).await?;
    
    // GetVersionを呼び出し
    let request = tonic::Request::new(Empty {});
    let response = client.get_version(request).await?;
    
    println!("Server version: {}", response.into_inner().version);
    
    Ok(())
}
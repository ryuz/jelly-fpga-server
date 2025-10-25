#!/usr/bin/env python3

import sys
sys.path.insert(0, '../jelly-fpga-client-py')

from jelly_fpga_client import JellyFpgaClient

def main():
    target = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1:8051"
    print(f"Connecting to: {target}")
    
    # サーバーに接続
    client = JellyFpgaClient(target)
    
    # バージョンを取得
    version = client.get_version()
    print(f"Server version: {version}")

if __name__ == "__main__":
    main()
[English README.md is here](README.md)

# 概要

`jelly-fpga-server`は、Zynq/ZynqMP をターゲットとしたgRPCベースのサーバーアプリケーションです。主にZynq UltraScale+デバイス（Kria KV260など）での使用を想定しています。

root 権限が必要な PL(Programmable Logic) 部などの制御を gRPCサーバーとしてサービス起動しておくことで、FPGA開発を効率化します。

ボード内だけでなく、ネットワーク経由でもアクセス可能ですので、Vivado などの動作する高性能なPCからクロスコンパイルで開発する場合もシームレスな開発が可能となります。

なお、研究開発用途を想定しており、認証などの機能は組み込んでいないため、ローカルネットワークでの使用を推奨します。

# 環境

現状、以下の環境で動作確認を行っています。

- [Kria KV260](https://www.amd.com/ja/products/system-on-modules/kria/k26/kv260-vision-starter-kit.html) + [Ubuntu 24.04](https://ubuntu.com/download/amd)
- [ZYBO Z7-20](https://digilent.com/reference/programmable-logic/zybo-z7/) + [Debian12](https://github.com/ikwzm/FPGA-SoC-Debian12)

主に [jelly-fpga-loader](https://github.com/ryuz/jelly-fpga-loader) を使用したFPGAビットストリームダウンロードや、各種言語バインディングからのFPGA操作を想定しています。

- Rust : [jelly-fpga-client-rs](https://github.com/ryuz/jelly-fpga-client-rs)
- Python : [jelly-fpga-client-py](https://github.com/ryuz/jelly-fpga-client-py)
- Elixir : [jelly-fpga-client-ex](https://github.com/ryuz/jelly-fpga-client-ex)


# インストール

サーバーをインストールするFPGAボード側の Linux にて、以下の手順でインストールを行います。

## 事前準備

dtc (Device Tree Compiler) と bootgen を利用しますので、以下のコマンドでインストールしてください。

```bash
sudo apt update
sudo apt install libssl-dev dtc
```

```bash
git clone https://github.com/Xilinx/bootgen
cd bootgen/
make
sudo cp bootgen /usr/local/bin/
```

## バイナリインストール

以下のコマンドを実行すると、github のリリースページから最新のバイナリをダウンロードしてインストールします。

```bash
curl -LsSf https://raw.githubusercontent.com/ryuz/jelly-fpga-server/master/binst.sh | sudo bash
```

## ソースコードからビルドしてインストール

Rust のインストールが必要です。以下のコマンドでインストールしてください。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Protocol Buffers コンパイラ (protoc) のインストールも必要です。以下のコマンドでインストールしてください。

```bash
sudo apt update
sudo apt install protobuf-compiler
```

次に、リポジトリをクローンしてインストールスクリプトを実行します。

```bash
git clone https://github.com/ryuz/jelly-fpga-server.git
cd jelly-fpga-server
sudo ./install.sh
```

## アンインストール

以下のコマンドでアンインストールを行います。

```bash
git clone https://github.com/ryuz/jelly-fpga-server.git
cd jelly-fpga-server
sudo ./uninstall.sh
```

もしくは手動で以下のようにアンインストールしてください。

```bash
sudo systemctl stop jelly-fpga-server
sudo systemctl disable jelly-fpga-server
sudo rm /usr/local/bin/jelly-fpga-server
sudo rm /etc/systemd/system/jelly-fpga-server.service
sudo rm /etc/default/jelly-fpga-server
```

## 使用方法

サービスとしての起動：

```bash
sudo systemctl start jelly-fpga-server
```

サービスの状態確認：

```bash
sudo systemctl status jelly-fpga-server
```

サービスの停止：

```bash
sudo systemctl stop jelly-fpga-server
```


## 環境変数設定ファイル

`/etc/default/jelly-fpga-server` に環境変数を設定することで、サービス起動時のオプションを指定できます。

```bash
オプション:
  -v, --verbose <VERBOSE>  詳細レベル（0-2）[default: 0]
      --external           外部接続を許可
  -p, --port <PORT>        リスニングポート [default: 8051]
      --allow-sudo         sudo権限での実行を許可
  -h, --help               ヘルプメッセージを表示
  -V, --version            バージョン情報を表示
```

デフォルトで --external を有効化しており、外部からの接続を許可しています。必要に応じて変更してください。




# gRPC経由でリモート実行できる主な機能

## 機能概要

- FPGAにビットストリームを読み込み
- デバイスツリーオーバーレイの適用
- xmutil や dfx-mgr-client で管理するアクセラレータの登録・削除
- メモリマップドI/Oアクセス
- Userspace I/Oデバイスアクセス
- [u-dma-buf](https://github.com/ikwzm/udmabuf) へのアクセス


## API仕様

サーバーは以下のgRPCサービスを提供します：

### FPGAコントロール
- `Reset`: システムリセット
- `Load`: ビットストリームの読み込み
- `Unload`: ビットストリームのアンロード

### ファームウェア管理
- `UploadFirmware`: ファームウェアのアップロード（ストリーミング）
- `RemoveFirmware`: ファームウェアの削除
- `LoadBitstream`: ビットストリームの読み込み
- `LoadDtbo`: デバイスツリーオーバーレイの読み込み

### メモリアクセサ
- `OpenMmap`: メモリマップドアクセサの作成
- `OpenUio`: UIOアクセサの作成
- `OpenUdmabuf`: UDMABUFアクセサの作成
- `Subclone`: サブアクセサの作成
- `Close`: アクセサのクローズ

### メモリ操作
- `WriteMemU/I`: メモリへの整数書き込み
- `ReadMemU/I`: メモリからの整数読み込み
- `WriteRegU/I`: レジスタへの整数書き込み
- `ReadRegU/I`: レジスタからの整数読み込み
- `WriteMemF32/F64`: メモリへの浮動小数点書き込み
- `ReadMemF32/F64`: メモリからの浮動小数点読み込み
- `MemCopyTo/From`: バイト配列のコピー

詳細なAPI仕様は`protos/jelly_fpga_control.proto`を参照してください。


## 関連プロジェクト

- [jelly-fpgautil-rs](https://github.com/ryuz/jelly-fpgautil-rs) - FPGA制御ユーティリティ
- [jelly-uidmng-rs](https://github.com/ryuz/jelly-uidmng-rs) - UID管理ユーティリティ
- [jelly-mem_access](https://crates.io/crates/jelly-mem_access) - メモリアクセスライブラリ


## 関連記事

本ソフトに関係する記事が下記にあります。

- [jelly-fpga-server の解説記事（Zenn）](https://zenn.dev/ryuz88/articles/jelly-fpga-server)


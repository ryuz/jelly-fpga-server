# jelly-fpga-server

Jelly FPGAプロジェクト用のgRPCサーバーです。FPGA制御、メモリアクセス、ファームウェア管理機能を提供します。

## 概要

`jelly-fpga-server`は、FPGAデバイスの制御とメモリアクセスを行うためのgRPCベースのサーバーアプリケーションです。主にZynq UltraScale+デバイス（Kria KV260など）での使用を想定しています。

## 主な機能

### FPGA制御
- **ビットストリームの読み込み**: FPGAにビットストリームを読み込み
- **DTBOの読み込み**: デバイスツリーオーバーレイの適用
- **ファームウェア管理**: ファームウェアのアップロード・削除

### メモリアクセス
- **MMAP**: メモリマップドI/Oアクセス
- **UIO**: Userspace I/Oデバイスアクセス
- **UDMABUF**: DMAバッファアクセス

### データ型サポート
- 符号なし整数（u8, u16, u32, u64, usize）
- 符号付き整数（i8, i16, i32, i64, isize）
- 浮動小数点数（f32, f64）
- バイト配列のコピー

## ビルド要件

- Rust 1.70+
- Protocol Buffers compiler (protoc)
- cargo


Protocol Buffers compiler は以下のコマンドでインストールできます

```bash
sudo apt update 
sudo apt install -y protobuf-compiler
```

### クロスコンパイル要件（Kria KV260向け）
- [cross](https://github.com/cross-rs/cross)


## バイナリインストール

```bash
curl -sL https://raw.githubusercontent.com/ryuz/jelly-fpga-server/master/binst.sh | bash
```


## インストール

### 1. リポジトリのクローン
```bash
git clone https://github.com/ryuz/jelly-fpga-server.git
cd jelly-fpga-server
```

### 2. ビルド

#### ローカル環境でのビルド
```bash
cargo build --release
```

#### Makefileを使用したビルド
```bash
make build
```

#### Kria KV260向けクロスコンパイル
```bash
# crossツールのインストール（初回のみ）
cargo install cross

# クロスコンパイル
make kria-build
```

## 使用方法

### 基本的な実行
```bash
./target/release/jelly-fpga-server
```

### コマンドラインオプション

```bash
jelly-fpga-server [OPTIONS]

オプション:
  -v, --verbose <VERBOSE>  詳細レベル（0-2）[default: 0]
      --external           外部接続を許可
  -p, --port <PORT>        リスニングポート [default: 8051]
      --allow-sudo         sudo権限での実行を許可
  -h, --help               ヘルプメッセージを表示
  -V, --version            バージョン情報を表示
```

### 使用例

#### 1. ローカルホストでの実行（デフォルト）
```bash
./target/release/jelly-fpga-server
```

#### 2. 外部からの接続を許可して実行
```bash
./target/release/jelly-fpga-server --external
```

#### 3. 詳細ログ出力で実行
```bash
./target/release/jelly-fpga-server --verbose 2
```

#### 4. sudo権限を許可して実行
```bash
sudo ./target/release/jelly-fpga-server --allow-sudo --external
```

### Kria KV260での実行

#### 1. ファイルのコピーと実行
```bash
# KRIA_BOARD_ADDRESS環境変数を設定
export KRIA_BOARD_ADDRESS=user@your-kria-ip

# ビルドとデプロイ
make kria-run
```

#### 2. 手動でのデプロイ
```bash
# クロスコンパイル
make kria-build

# Kriaボードにコピー
scp target/aarch64-unknown-linux-gnu/release/jelly-fpga-server user@kria-ip:/tmp/

# Kriaボードで実行
ssh user@kria-ip
sudo /tmp/jelly-fpga-server --external --allow-sudo
```

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

## セキュリティ

- デフォルトでは`127.0.0.1`（ローカルホスト）のみでリスニング
- `--external`オプションで外部接続を許可
- `--allow-sudo`オプションで管理者権限での操作を許可
- ファームウェアファイルは`/lib/firmware/`ディレクトリに制限

## 依存関係

主な依存関係：
- `tonic`: gRPCフレームワーク
- `tokio`: 非同期ランタイム
- `prost`: Protocol Buffersライブラリ
- `clap`: コマンドライン引数パーサー
- `jelly-fpgautil`: FPGA制御ユーティリティ
- `jelly-uidmng`: UID管理ユーティリティ
- `jelly-mem_access`: メモリアクセスライブラリ

## ライセンス

このプロジェクトのライセンス情報については、LICENSEファイルを参照してください。

## 貢献

バグ報告や機能要求は、GitHubのIssueページでお願いします。プルリクエストも歓迎します。

## 関連プロジェクト

- [jelly-fpgautil-rs](https://github.com/ryuz/jelly-fpgautil-rs) - FPGA制御ユーティリティ
- [jelly-uidmng-rs](https://github.com/ryuz/jelly-uidmng-rs) - UID管理ユーティリティ
- [jelly-mem_access](https://crates.io/crates/jelly-mem_access) - メモリアクセスライブラリ

## 解説記事

詳細な使用方法や技術解説については、以下の記事を参照してください：

- [jelly-fpga-server の解説記事（Zenn）](https://zenn.dev/ryuz88/articles/jelly-fpga-server)
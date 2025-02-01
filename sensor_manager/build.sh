#!/bin/bash

# ビルドを行うシェルスクリプト
# filepath: /home/srs1123/light_control_app/build.sh

# 現在のビルド成果物を削除
rm -rf output

# ビルドの前に必要なターゲットを追加
rustup target add aarch64-unknown-linux-gnu

# 依存関係をクリーン
cargo clean

# ビルドを実行
cargo build --release --target=aarch64-unknown-linux-gnu

# 成果物をコピーするディレクトリを作成
mkdir -p output

# ビルド成果物をコピー
cp target/aarch64-unknown-linux-gnu/release/sensor_manager output/

echo "Build completed."
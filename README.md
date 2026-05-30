# DP100 Controller GUI

Alientek DP100 USB電源のコントローラーGUIアプリケーションです。
[Tauri](https://tauri.app/) + Rust + HTML/CSS/JavaScript で実装しています。

## 機能

- リアルタイム電圧・電流・消費電力・温度表示
- プロファイル管理（0〜9の10個）
- 電圧・電流・OVP・OCP設定
- OUTPUT ON/OFF制御
- リアルタイムグラフ（Vout/Iout）
- OUTPUT ON中のプロファイル切り替え禁止
- 再接続時のOUTPUT状態復元

## 対応デバイス

| 項目 | 値 |
|------|-----|
| デバイス名 | ALIENTEK ATK-MDP100 |
| Vendor ID | `0x2E3C` |
| Product ID | `0xAF01` |

## 動作環境

- Linux (Debian/Ubuntu/Arch 等)
- Windows 10/11

## ビルド要件

- Rust 1.70+
- Node.js 20+
- Tauri CLI 2.x

### Linux追加依存パッケージ (Debian/Ubuntu)

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential \
  curl wget file libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev libudev-dev
```

## セットアップ

### 1. リポジトリのクローン

```bash
git clone https://github.com/ユーザー名/dp100-gui.git
cd dp100-gui
```

### 2. udevルールの追加 (Linuxのみ)

```bash
sudo nano /etc/udev/rules.d/99-dp100.rules
```

以下を追加:

```
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="2e3c", ATTRS{idProduct}=="af01", MODE="0666"
```

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### 3. 開発サーバー起動

```bash
cargo tauri dev
```

### 4. リリースビルド

```bash
cargo tauri build
```

成果物:

```
src-tauri/target/release/bundle/
├── appimage/dp100-gui_x.x.x_amd64.AppImage  # Linux (推奨)
├── deb/dp100-gui_x.x.x_amd64.deb            # Debian/Ubuntu
└── rpm/dp100-gui-x.x.x.x86_64.rpm           # Fedora/RHEL
```

## 使い方

1. DP100をUSBケーブルで接続（USBDモード: `◀` を2回タップで切替）
2. アプリを起動
3. 「接続」ボタンをクリック
4. プロファイルを選択して電圧・電流を設定
5. 「OUTPUT OFF」ボタンで出力ON

> **注意**: OUTPUT ON中はプロファイルの切り替えおよび設定値の変更はできません。

## プロトコル

USB HID (Vendor Defined) で通信します。

| コマンド | 内容 |
|---------|------|
| `0x10` | デバイス情報取得 |
| `0x30` | 電圧・電流・温度取得 |
| `0x35` | プロファイル読み書き・OUTPUT ON/OFF |

チェックサム: CRC-16 Modbus (Little Endian)

プロトコルの詳細は [weigu1/dp100_manipulator](https://github.com/weigu1/dp100_manipulator) を参照。

## 技術スタック

| 役割 | 技術 |
|------|------|
| フレームワーク | Tauri 2.x |
| バックエンド | Rust |
| HID通信 | hidapi 2.x |
| フロントエンド | HTML / CSS / JavaScript |
| グラフ | Chart.js 4.x |

## ライセンス

MIT

use crate::crc16;
use hidapi::HidApi;

const VID: u16 = 0x2E3C;
const PID: u16 = 0xAF01;

const PKT_SIZE: usize = 64;
const CMD_DEVICE_INFO: u8 = 0x10;
const CMD_BASIC_INFO: u8 = 0x30;
const CMD_BASIC_SET: u8 = 0x35;

pub struct Dp100 {
    device: hidapi::HidDevice,
}

impl Dp100 {
    pub fn open() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| e.to_string())?;
        let device = api.open(VID, PID).map_err(|e| e.to_string())?;
        Ok(Self { device })
    }

    fn send(&self, cmd: u8, data: &[u8]) -> Result<(), String> {
        let mut pkt = [0u8; PKT_SIZE + 1];
        pkt[0] = 0x00;
        pkt[1] = 0xFB;
        pkt[2] = cmd;
        pkt[3] = 0x00;
        pkt[4] = data.len() as u8;

        for (i, b) in data.iter().enumerate() {
            pkt[5 + i] = *b;
        }

        let crc = crc16::calc(&pkt[1..5 + data.len()]);
        pkt[5 + data.len()] = (crc & 0xFF) as u8;
        pkt[5 + data.len() + 1] = (crc >> 8) as u8;

        self.device.write(&pkt).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn recv(&self, cmd: u8) -> Result<Vec<u8>, String> {
        let mut buf = [0u8; PKT_SIZE];

        self.device
            .read_timeout(&mut buf, 1000)
            .map_err(|e| e.to_string())?;

        if buf[0] != 0xFA {
            return Err(format!("不正なヘッダ: 0x{:02X}", buf[0]));
        }
        if buf[1] != cmd {
            return Err(format!(
                "コマンド不一致: 期待 0x{:02X} 実際 0x{:02X}",
                cmd, buf[1]
            ));
        }

        let len = buf[3] as usize;

        let crc_calc = crc16::calc(&buf[0..4 + len]);
        let crc_recv = (buf[4 + len] as u16) | ((buf[5 + len] as u16) << 8);
        if crc_calc != crc_recv {
            return Err(format!(
                "CRCエラー: 計算値 0x{:04X} 受信値 0x{:04X}",
                crc_calc, crc_recv
            ));
        }

        Ok(buf[4..4 + len].to_vec())
    }

    // デバイス情報取得 (0x10)
    pub fn get_device_info(&self) -> Result<String, String> {
        self.send(CMD_DEVICE_INFO, &[])?;
        let data = self.recv(CMD_DEVICE_INFO)?;
        // Byte0〜15: デバイス名 ASCII
        let name = String::from_utf8_lossy(&data[0..16])
            .trim_matches('\0')
            .to_string();
        Ok(name)
    }

    // 基本情報取得 (0x30)
    pub fn get_basic_info(&self) -> Result<BasicInfo, String> {
        self.send(CMD_BASIC_INFO, &[])?;
        let data = self.recv(CMD_BASIC_INFO)?;

        if data.len() < 8 {
            return Err(format!("データ不足: {} bytes", data.len()));
        }

        Ok(BasicInfo {
            vin: u16::from_le_bytes([data[0], data[1]]) as f64 / 100.0,
            vout: u16::from_le_bytes([data[2], data[3]]) as f64 / 100.0,
            iout: u16::from_le_bytes([data[4], data[5]]) as f64 / 1000.0,
            temp: u16::from_le_bytes([data[6], data[7]]) as f64 / 10.0,
        })
    }

    // プロファイル取得 (index: 0〜9, 0xFFでアクティブ)
    pub fn get_profile(&self, index: u8) -> Result<Profile, String> {
        let cmd_byte = if index == 0xFF {
            0x80
        } else {
            0x80 | (index & 0x0F)
        };
        self.send(CMD_BASIC_SET, &[cmd_byte])?;
        let data = self.recv(CMD_BASIC_SET)?;

        if data.len() < 10 {
            return Err(format!("データ不足: {} bytes", data.len()));
        }

        Ok(Profile {
            index: data[0],
            output_on: data[1] == 0x01,
            vset: u16::from_le_bytes([data[2], data[3]]) as f64 / 100.0,
            iset: u16::from_le_bytes([data[4], data[5]]) as f64 / 1000.0,
            ovp: u16::from_le_bytes([data[6], data[7]]) as f64 / 100.0,
            ocp: u16::from_le_bytes([data[8], data[9]]) as f64 / 1000.0,
        })
    }

    // プロファイル書き込み (0x35 / 0x4X)
    pub fn set_profile(&self, profile: &Profile) -> Result<(), String> {
        let vset = (profile.vset * 100.0) as u16;
        let iset = (profile.iset * 1000.0) as u16;
        let ovp = (profile.ovp * 100.0) as u16;
        let ocp = (profile.ocp * 1000.0) as u16;

        let data = [
            0x40 | (profile.index & 0x0F), // 書き込みコマンド
            profile.output_on as u8,
            (vset & 0xFF) as u8,
            (vset >> 8) as u8,
            (iset & 0xFF) as u8,
            (iset >> 8) as u8,
            (ovp & 0xFF) as u8,
            (ovp >> 8) as u8,
            (ocp & 0xFF) as u8,
            (ocp >> 8) as u8,
        ];

        self.send(CMD_BASIC_SET, &data)?;
        self.recv(CMD_BASIC_SET)?;
        Ok(())
    }

    // 出力ON/OFF (0x35 / 0x2X)
    pub fn set_output(&self, index: u8, on: bool) -> Result<(), String> {
        let data = [0x20 | (index & 0x0F), on as u8];
        self.send(CMD_BASIC_SET, &data)?;
        self.recv(CMD_BASIC_SET)?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BasicInfo {
    pub vin: f64,
    pub vout: f64,
    pub iout: f64,
    pub temp: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub index: u8,
    pub output_on: bool,
    pub vset: f64,
    pub iset: f64,
    pub ovp: f64,
    pub ocp: f64,
}

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

    // バッファをフラッシュする
    fn flush(&self) {
        let mut buf = [0u8; 64];
        while self.device.read_timeout(&mut buf, 0).unwrap_or(0) > 0 {}
    }

    fn send(&self, cmd: u8, data: &[u8]) -> Result<(), String> {
        self.flush(); // 送信前にフラッシュ
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
        // ASCII文字のみ抽出
        let name: String = data[0..16]
            .iter()
            .filter(|&&b| b.is_ascii_graphic() || b == b' ')
            .map(|&b| b as char)
            .collect();
        Ok(name)
    }

    // 基本情報取得 (0x30)
    pub fn get_basic_info(&self) -> Result<BasicInfo, String> {
        self.send(CMD_BASIC_INFO, &[])?;
        let data = self.recv(CMD_BASIC_INFO)?;
        //log::info!("basic_info full: {:02X?}", &data);
        if data.len() < 15 {
            return Err(format!("データ不足: {} bytes", data.len()));
        }

        Ok(BasicInfo {
            vin: u16::from_le_bytes([data[0], data[1]]) as f64 / 1000.0,
            vout: u16::from_le_bytes([data[2], data[3]]) as f64 / 1000.0,
            iout: u16::from_le_bytes([data[4], data[5]]) as f64 / 1000.0,
            temp: u16::from_le_bytes([data[6], data[7]]) as f64 / 1000.0,
            output_on: data[14] == 0x01, // 0x01=ON, 0x02=OFF
        })
    }

    // プロファイル取得 (index: 0〜9, 0xFFでアクティブ)
    pub fn get_profile(&self, index: u8) -> Result<Profile, String> {
        self.send(CMD_BASIC_SET, &[index])?;
        let data = self.recv(CMD_BASIC_SET)?;

        //log::info!(
        //    "get_profile({}) raw: {:02X?}",
        //    index,
        //    &data[0..10.min(data.len())]
        //);

        if data.len() < 10 {
            return Err(format!("データ不足: {} bytes", data.len()));
        }

        Ok(Profile {
            index: data[0],
            output_on: data[1] == 0x01,
            vset: u16::from_le_bytes([data[2], data[3]]) as f64 / 1000.0, // 100.0→1000.0
            iset: u16::from_le_bytes([data[4], data[5]]) as f64 / 1000.0,
            ovp: u16::from_le_bytes([data[6], data[7]]) as f64 / 1000.0, // 100.0→1000.0
            ocp: u16::from_le_bytes([data[8], data[9]]) as f64 / 1000.0,
        })
    }

    // プロファイル書き込み (0x35 / 0x4X)
    pub fn set_profile(&self, profile: &Profile) -> Result<(), String> {
        let vset = (profile.vset * 1000.0) as u16;
        let iset = (profile.iset * 1000.0) as u16;
        let ovp = (profile.ovp * 1000.0) as u16;
        let ocp = (profile.ocp * 1000.0) as u16;

        let data = [
            0x40 | (profile.index & 0x0F),
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
        //log::info!("set_profile({}) data: {:02X?}", profile.index, &data);
        self.send(CMD_BASIC_SET, &data)?;
        //let resp = self.recv(CMD_BASIC_SET)?;
        //log::info!("set_profile response: {:02X?}", &resp[0..4.min(resp.len())]);
        Ok(())
    }

    // 出力ON/OFF (0x35 / 0x2X)
    pub fn set_output(&self, index: u8, on: bool) -> Result<(), String> {
        // まずプロファイル情報を取得
        let profile = self.get_profile(index)?;
        let vset = (profile.vset * 1000.0) as u16;
        let iset = (profile.iset * 1000.0) as u16;
        let ovp = (profile.ovp * 1000.0) as u16;
        let ocp = (profile.ocp * 1000.0) as u16;

        let data = [
            0x20 | (index & 0x0F), // ON/OFFコマンド
            on as u8,              // ON=1 / OFF=0
            (vset & 0xFF) as u8,
            (vset >> 8) as u8,
            (iset & 0xFF) as u8,
            (iset >> 8) as u8,
            (ovp & 0xFF) as u8,
            (ovp >> 8) as u8,
            (ocp & 0xFF) as u8,
            (ocp >> 8) as u8,
        ];
        //log::info!("set_output data: {:02X?}", &data);
        self.send(CMD_BASIC_SET, &data)?;
        //let resp = self.recv(CMD_BASIC_SET)?;
        //log::info!("set_output response: {:02X?}", &resp[0..4.min(resp.len())]);
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BasicInfo {
    pub vin: f64,
    pub vout: f64,
    pub iout: f64,
    pub temp: f64,
    pub output_on: bool,
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

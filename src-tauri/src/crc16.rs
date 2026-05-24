// CRC-16 Modbus
// 初期値: 0xFFFF
// 多項式: 0xA001 (0x8005の逆順)
pub fn calc(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for byte in data {
        crc ^= *byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

// テスト
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc_hello_command() {
        // FB 10 00 00 → CRC = 0xC530 (実機プロトコル確認済み)
        let data = [0xFBu8, 0x10, 0x00, 0x00];
        let crc = calc(&data);
        assert_eq!(crc, 0xC530);
    }

    #[test]
    fn test_crc_basic_info_command() {
        // FB 30 00 00 → CRC = 0x0F31 (実機接続後に確認)
        let data = [0xFBu8, 0x30, 0x00, 0x00];
        let crc = calc(&data);
        assert_eq!(crc, 0x0F31);
    }
}

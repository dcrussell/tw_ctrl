//! This module provides an implementation of CRC16 XMODEM

///Calculate the Checksum for an arr of bytes
pub fn crc16(arr: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    let mut it = arr.iter();
    let mut sz: usize = arr.len();
    while 0 != sz {
        let data: u16 = *it.next().unwrap() as u16;
        crc = crc ^ (data << 8);
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
        sz -= 1;
    }
    crc
}

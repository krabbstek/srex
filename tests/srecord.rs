use srex::srecord::calculate_checksum;

#[test]
fn test_calculate_checksum() {
    assert_eq!(calculate_checksum(0x13, 0x7AF0, &[0x0A, 0x0A, 0x0D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), 0x61);
    assert_eq!(calculate_checksum(0x0F, 0x0000, &[0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x20, 0x20, 0x20, 0x20, 0, 0]), 0x3C);
    assert_eq!(calculate_checksum(0x1F, 0x0000, &[0x7C, 0x08, 0x02, 0xA6, 0x90, 0x01, 0x00, 0x04, 0x94, 0x21, 0xFF,
        0xF0, 0x7C, 0x6C, 0x1B, 0x78, 0x7C, 0x8C, 0x23, 0x78, 0x3C, 0x60, 0x00, 0x00, 0x38, 0x63, 0x00, 0x00]), 0x26);
    assert_eq!(calculate_checksum(0x1F, 0x001C, &[0x4B, 0xFF, 0xFF, 0xE5, 0x39, 0x80, 0x00, 0x00, 0x7D, 0x83, 0x63,
        0x78, 0x80, 0x01, 0x00, 0x14, 0x38, 0x21, 0x00, 0x10, 0x7C, 0x08, 0x03, 0xA6, 0x4E, 0x80, 0x00, 0x20]), 0xE9);
    assert_eq!(calculate_checksum(0x11, 0x0038, &[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
        0x2E, 0x0A, 0x00]), 0x42);
    assert_eq!(calculate_checksum(0x03, 0x0003, &[]), 0xF9);
    assert_eq!(calculate_checksum(0x03, 0x0000, &[]), 0xFC);
}

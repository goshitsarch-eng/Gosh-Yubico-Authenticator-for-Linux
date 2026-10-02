use gosh_authenticator_core::qr::scan_qr_file;
use image::Luma;

#[test]
fn real_qr_image_decodes_unicode_label_and_custom_period() {
    let uri = "otpauth://totp/%E6%9C%8D%E5%8A%A1:%C3%A9?secret=JBSWY3DPEHPK3PXP&period=60&digits=8&algorithm=SHA256";
    let code = qrcode::QrCode::new(uri).unwrap();
    let image = code.render::<Luma<u8>>().min_dimensions(400, 400).build();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("QR image 配置 with spaces.png");
    image.save(&path).unwrap();
    let credential = scan_qr_file(&path).unwrap();
    assert_eq!(credential.issuer.as_deref(), Some("服务"));
    assert_eq!(credential.account, "é");
    assert_eq!(credential.period, 60);
    assert_eq!(credential.digits, 8);
}

#[test]
fn unreadable_corrupt_and_oversized_images_report_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.png");
    assert!(scan_qr_file(&path).is_err());
    std::fs::write(&path, b"not a PNG").unwrap();
    assert!(scan_qr_file(&path).is_err());
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(16 * 1024 * 1024 + 1).unwrap();
    assert!(scan_qr_file(&path)
        .unwrap_err()
        .to_string()
        .contains("16 MiB"));
}

#[test]
fn desktop_icons_have_transparent_edges_and_no_dark_matte() {
    for bytes in [
        include_bytes!("../icons/32x32.png").as_slice(),
        include_bytes!("../icons/128x128.png").as_slice(),
        include_bytes!("../icons/128x128@2x.png").as_slice(),
    ] {
        let image = tauri::image::Image::from_bytes(bytes).unwrap();
        let rgba = image.rgba();
        assert_eq!(rgba[3], 0, "corner must be transparent");
        let opaque: Vec<_> = rgba
            .chunks_exact(4)
            .filter(|pixel| pixel[3] == 255)
            .collect();
        assert!(!opaque.is_empty());
        assert!(
            opaque.iter().all(|pixel| pixel[..3] == [146, 219, 181]),
            "opaque pixels must be the uniform mint brand color, not a dark background/halo"
        );
    }
}

#[test]
fn native_window_icon_is_the_current_bundled_brand() {
    let actual = crate::window_brand_icon().unwrap();
    let expected = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png")).unwrap();
    assert_eq!(actual.rgba(), expected.rgba());
    assert_eq!(actual.width(), 128);
    assert_eq!(actual.height(), 128);
    let source = include_str!("lib.rs");
    assert!(source.contains("let _ = window.set_icon(icon)"));
}

#[test]
fn executable_installer_and_uninstaller_use_the_same_icon() {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    let bundle = &config["bundle"];
    assert!(bundle["icon"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "icons/icon.ico"));
    assert_eq!(bundle["windows"]["nsis"]["installerIcon"], "icons/icon.ico");
    assert_eq!(
        bundle["windows"]["nsis"]["uninstallerIcon"],
        "icons/icon.ico"
    );
    let ico = include_bytes!("../icons/icon.ico");
    assert_eq!(&ico[..4], &[0, 0, 1, 0]);
    let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;
    let sizes: Vec<_> = (0..count).map(|index| ico[6 + index * 16]).collect();
    assert!(sizes.contains(&16), "small taskbar/titlebar frame");
    assert!(sizes.contains(&0), "256px Windows icon frame");
}

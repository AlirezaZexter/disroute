fn main() {
    let attributes = tauri_build::Attributes::new();
    // Only packaged Windows applications request elevation. Development and
    // unit tests retain Tauri's default manifest.
    let attributes = if cfg!(target_os = "windows")
        && std::env::var_os("CARGO_FEATURE_CUSTOM_PROTOCOL").is_some()
    {
        attributes.windows_attributes(
            tauri_build::WindowsAttributes::new()
                .app_manifest(include_str!("windows/app.manifest")),
        )
    } else {
        attributes
    };
    tauri_build::try_build(attributes).expect("failed to prepare application resources");
}

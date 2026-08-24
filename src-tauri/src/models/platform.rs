pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
}

pub fn parse_os(os: String) -> String {
    os.to_lowercase().replace("darwin", "osx")
}

/// Returns the current operating system that launcher is being launched with.
///
/// Such as "osx", "linux", "windows"
pub fn get_current_os() -> String {
    parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}
pub fn get_current_os_with_architecture() -> String {
    let os = get_current_os();

    match os.as_str() {
        "windows" => {
            #[cfg(target_arch = "x86")]
            {
                "windows-x86".to_string()
            }
            #[cfg(target_arch = "x86_64")]
            {
                "windows-x64".to_string()
            }
            #[cfg(target_arch = "aarch64")]
            {
                "windows-arm64".to_string()
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
            {
                "windows-x64".to_string() // safe fallback
            }
        }
        "osx" => {
            #[cfg(target_arch = "aarch64")]
            {
                "mac-os-arm64".to_string()
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                "mac-os".to_string()
            }
        }
        "linux" | _ => {
            #[cfg(target_arch = "x86")]
            {
                "linux-i386".to_string()
            }
            #[cfg(not(target_arch = "x86"))]
            {
                "linux".to_string()
            }
        }
    }
}
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
pub fn get_current_os_with_architecture() -> String{
    let mut os = parse_os(sys_info::os_release().expect("Unsupported Operating System"));
     if os == "windows" {
         "windows-x64".to_string() //FOR NOW "
    }else if os == "linux" {
         "linux".to_string()  // FOR NOW
    }else {
         "mac-os".to_string() // FOR NOW
    }
}
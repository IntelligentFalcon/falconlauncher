pub enum OperatingSystem {
    Windows,
    Linux,
    MacOS,
}

pub fn parse_os(os: String) -> String {
    os.to_lowercase().replace("darwin", "osx")
}

pub fn get_current_os() -> String {
    parse_os(sys_info::os_type().expect("Unsupported Operating System"))
}
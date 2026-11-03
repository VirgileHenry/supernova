pub fn vulkan_name_to_string(name: &[i8]) -> String {
    /* Convert the &[i8] slice to a Vec<u8> owned storage for the String */
    let name_bytes: Vec<u8> = name.iter().take_while(|&&byte| byte != 0).map(|&byte| byte as u8).collect();
    /* UTF-8 encoding check */
    match String::from_utf8(name_bytes) {
        Ok(name) => name,
        Err(e) => format!("Unknown device name: {e}"),
    }
}

pub fn format_api_version(version: u32) -> String {
    format!(
        "{}.{}.{}",
        ash::vk::api_version_major(version),
        ash::vk::api_version_minor(version),
        ash::vk::api_version_patch(version),
    )
}

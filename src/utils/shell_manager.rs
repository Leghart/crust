pub struct ShellManager {}

impl ShellManager {
    pub fn is_background_mode() -> bool {
        ShellManager::is_bool_flag_set("CRUST_BG_MODE")
    }

    pub fn is_shell_invoke() -> bool {
        ShellManager::is_bool_flag_set("CRUST_SHELL_INVOKE")
    }

    fn is_bool_flag_set(flag: &str) -> bool {
        std::env::var(flag).map_or(false, |v| v.to_lowercase() == "true")
    }
}

// Application constants

/// Application name
pub const NAME: &str = "DevKit";

/// Application name (lowercase)
pub const NAME_EN: &str = "devkit";

/// Application description
pub const DESCRIPTION: &str = "Development toolkit for enhanced coding workflow";

/// 应用程序版本（从 Cargo.toml 获取）
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application author
pub const AUTHOR: &str = "aspect-build";

/// Application homepage
pub const HOMEPAGE: &str = "https://github.com/aspect-build/devkit";

/// Application repository
pub const REPOSITORY: &str = "https://github.com/aspect-build/devkit";

/// Application license
pub const LICENSE: &str = "MIT";

/// Config file name
pub const CONFIG_FILE_NAME: &str = "config.json";

/// Log file prefix
pub const LOG_FILE_PREFIX: &str = "devkit";

/// Application identifier
pub const APP_IDENTIFIER: &str = "com.aspect.devkit";

/// 用户代理字符串
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

// 防误触退出相关常量
/// 退出确认时间窗口（秒）
pub const EXIT_CONFIRMATION_WINDOW_SECS: u64 = 3;

/// 需要的连续退出尝试次数
pub const REQUIRED_EXIT_ATTEMPTS: u32 = 2;

// 应用程序信息结构体
#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub name_en: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub homepage: String,
    pub repository: String,
    pub license: String,
    pub identifier: String,
    pub user_agent: String,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: NAME.to_string(),
            name_en: NAME_EN.to_string(),
            description: DESCRIPTION.to_string(),
            version: VERSION.to_string(),
            author: AUTHOR.to_string(),
            homepage: HOMEPAGE.to_string(),
            repository: REPOSITORY.to_string(),
            license: LICENSE.to_string(),
            identifier: APP_IDENTIFIER.to_string(),
            user_agent: USER_AGENT.to_string(),
        }
    }
}

impl AppInfo {
    /// 获取完整的应用程序标题
    pub fn get_full_title(&self) -> String {
        format!("{} v{}", self.name, self.version)
    }

    /// 获取窗口标题
    pub fn get_window_title(&self) -> String {
        self.name.clone()
    }

    /// 转换为 JSON 格式
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "name_en": self.name_en,
            "description": self.description,
            "version": self.version,
            "author": self.author,
            "homepage": self.homepage,
            "repository": self.repository,
            "license": self.license,
            "identifier": self.identifier,
            "user_agent": self.user_agent
        })
    }
}

// 便捷函数
/// 获取应用程序信息
pub fn get_app_info() -> AppInfo {
    AppInfo::default()
}

/// 获取应用程序名称
pub fn get_app_name() -> &'static str {
    NAME
}

/// 获取应用程序版本
pub fn get_app_version() -> &'static str {
    VERSION
}

/// 获取用户代理字符串
pub fn get_user_agent() -> &'static str {
    USER_AGENT
}

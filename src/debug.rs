//! This module provides a development-only logging macro.
//!
//! 本模块提供了一个仅在开发时使用的日志宏。

/// A macro for logging development-only information.
///
/// This macro wraps `bevy::log::info!` and is only enabled when the `dev-logs` feature is active.
///
/// 用于记录仅开发信息的宏。
///
/// 此宏包装了 `bevy::log::info!`，仅在 `dev-logs` 功能激活时启用。
#[macro_export]
macro_rules! dev_info {
    ($($arg:tt)*) => {
        #[cfg(feature = "dev-logs")]
        {
            bevy::log::info!($($arg)*);
        }
    };
}

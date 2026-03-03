pub const MAX_MESSAGES: i64 = 1024;
pub const PAGE_SIZE: i64 = 20;
pub const MAX_PAGES: i64 = 51;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const VERSION_DATE: &str = "2026-03-04";

// 输入长度限制（按Unicode字符计数）
pub const MAX_MESSAGE_LENGTH: usize = 10_000;
pub const MAX_REPLY_LENGTH: usize = 5_000;
pub const MAX_TAG_NAME_LENGTH: usize = 50;
pub const MAX_SEARCH_LENGTH: usize = 200;

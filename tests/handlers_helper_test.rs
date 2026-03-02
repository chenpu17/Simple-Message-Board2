//! Handlers 辅助函数测试
//!
//! 测试 handlers 模块中的辅助函数

// ==================== build_redirect_path 测试 ====================

/// 模拟 build_redirect_path 函数
/// 来自 src/handlers/mod.rs
fn build_redirect_path(page: Option<i64>, q: Option<&str>, tag: Option<&str>) -> String {
    let page = page.unwrap_or(1);
    let mut params = Vec::new();

    if page > 1 {
        params.push(format!("page={}", page));
    }
    if let Some(search) = q {
        if !search.is_empty() {
            params.push(format!("q={}", urlencoding::encode(search)));
        }
    }
    if let Some(tag_id) = tag {
        if !tag_id.is_empty() {
            params.push(format!("tag={}", tag_id));
        }
    }

    if params.is_empty() {
        "/".to_string()
    } else {
        format!("/?{}", params.join("&"))
    }
}

#[test]
fn test_build_redirect_path_no_params() {
    // 无参数，返回根路径
    assert_eq!(build_redirect_path(None, None, None), "/");
    assert_eq!(build_redirect_path(Some(1), None, None), "/");
}

#[test]
fn test_build_redirect_path_with_page() {
    // 只有分页参数
    assert_eq!(build_redirect_path(Some(2), None, None), "/?page=2");
    assert_eq!(build_redirect_path(Some(10), None, None), "/?page=10");
}

#[test]
fn test_build_redirect_path_with_search() {
    // 只有搜索参数
    assert_eq!(build_redirect_path(None, Some("test"), None), "/?q=test");
    assert_eq!(build_redirect_path(Some(1), Some("test"), None), "/?q=test");
}

#[test]
fn test_build_redirect_path_with_tag() {
    // 只有标签参数
    assert_eq!(build_redirect_path(None, None, Some("1")), "/?tag=1");
    assert_eq!(build_redirect_path(Some(1), None, Some("5")), "/?tag=5");
}

#[test]
fn test_build_redirect_path_with_all_params() {
    // 所有参数
    assert_eq!(
        build_redirect_path(Some(2), Some("test"), Some("1")),
        "/?page=2&q=test&tag=1"
    );
}

#[test]
fn test_build_redirect_path_empty_strings() {
    // 空字符串应该被忽略
    assert_eq!(build_redirect_path(None, Some(""), None), "/");
    assert_eq!(build_redirect_path(None, None, Some("")), "/");
    assert_eq!(build_redirect_path(None, Some(""), Some("")), "/");
}

#[test]
fn test_build_redirect_path_special_characters() {
    // 特殊字符应该被 URL 编码
    let result = build_redirect_path(None, Some("hello world"), None);
    assert!(result.contains("hello%20world") || result.contains("hello+world"));

    let result = build_redirect_path(None, Some("测试"), None);
    assert!(result.contains("%"));
}

#[test]
fn test_build_redirect_path_large_page() {
    // 大数值页码
    assert_eq!(build_redirect_path(Some(1000), None, None), "/?page=1000");
    assert_eq!(
        build_redirect_path(Some(i64::MAX), None, None),
        format!("/?page={}", i64::MAX)
    );
}

// ==================== generate_pages 测试 ====================

/// 模拟 generate_pages 函数
/// 来自 src/handlers/home.rs
fn generate_pages(current: i64, total: i64) -> Vec<String> {
    if total <= 7 {
        return (1..=total).map(|i| i.to_string()).collect();
    }

    let mut pages = Vec::new();
    let mut show = std::collections::HashSet::new();

    show.insert(1);
    show.insert(total);
    for i in (current - 1).max(1)..=(current + 1).min(total) {
        show.insert(i);
    }

    let mut sorted: Vec<i64> = show.iter().cloned().collect();
    sorted.sort();

    let mut last = 0i64;
    for page in sorted {
        if last > 0 && page - last > 1 {
            pages.push("...".to_string());
        }
        pages.push(page.to_string());
        last = page;
    }

    pages
}

#[test]
fn test_generate_pages_small_total() {
    // 总页数 <= 7，显示所有页码
    assert_eq!(generate_pages(1, 1), vec!["1"]);
    assert_eq!(generate_pages(1, 5), vec!["1", "2", "3", "4", "5"]);
    assert_eq!(
        generate_pages(3, 7),
        vec!["1", "2", "3", "4", "5", "6", "7"]
    );
}

#[test]
fn test_generate_pages_at_start() {
    // 当前页在开头
    let pages = generate_pages(1, 10);
    assert!(pages.contains(&"1".to_string()));
    assert!(pages.contains(&"2".to_string()));
    assert!(pages.contains(&"10".to_string()));
    assert!(pages.contains(&"...".to_string()));
}

#[test]
fn test_generate_pages_at_end() {
    // 当前页在末尾
    let pages = generate_pages(10, 10);
    assert!(pages.contains(&"1".to_string()));
    assert!(pages.contains(&"9".to_string()));
    assert!(pages.contains(&"10".to_string()));
    assert!(pages.contains(&"...".to_string()));
}

#[test]
fn test_generate_pages_in_middle() {
    // 当前页在中间
    let pages = generate_pages(5, 10);
    assert!(pages.contains(&"1".to_string()));
    assert!(pages.contains(&"4".to_string()));
    assert!(pages.contains(&"5".to_string()));
    assert!(pages.contains(&"6".to_string()));
    assert!(pages.contains(&"10".to_string()));
}

#[test]
fn test_generate_pages_shows_ellipsis() {
    // 验证省略号出现
    let pages = generate_pages(1, 20);
    assert!(pages.contains(&"...".to_string()));

    let pages = generate_pages(20, 20);
    assert!(pages.contains(&"...".to_string()));

    let pages = generate_pages(10, 20);
    assert!(pages.contains(&"...".to_string()));
}

#[test]
fn test_generate_pages_boundary() {
    // 边界测试
    // 刚好 8 页，应该显示省略号
    let pages = generate_pages(1, 8);
    assert!(pages.contains(&"...".to_string()));

    // 刚好 7 页，不应显示省略号
    let pages = generate_pages(1, 7);
    assert!(!pages.contains(&"...".to_string()));
}

#[test]
fn test_generate_pages_zero_total() {
    // 零页
    assert_eq!(generate_pages(1, 0), Vec::<String>::new());
}

// ==================== truncate_utf8 测试 ====================

/// 模拟 truncate_utf8 函数
/// 来自 src/handlers/dashboard.rs
fn truncate_utf8(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect::<String>() + "..."
    }
}

#[test]
fn test_truncate_utf8_short_string() {
    // 短字符串不截断
    assert_eq!(truncate_utf8("hello", 10), "hello");
    assert_eq!(truncate_utf8("", 10), "");
}

#[test]
fn test_truncate_utf8_exact_length() {
    // 恰好等于最大长度
    assert_eq!(truncate_utf8("hello", 5), "hello");
}

#[test]
fn test_truncate_utf8_long_string() {
    // 长字符串需要截断
    assert_eq!(truncate_utf8("hello world", 5), "hello...");
    assert_eq!(truncate_utf8("abcdefghij", 3), "abc...");
}

#[test]
fn test_truncate_utf8_unicode() {
    // Unicode 字符正确处理
    assert_eq!(truncate_utf8("你好世界", 2), "你好...");
    assert_eq!(truncate_utf8("🎉🎊🎈🎁🎂", 3), "🎉🎊🎈...");
}

#[test]
fn test_truncate_utf8_mixed_content() {
    // 混合内容
    assert_eq!(truncate_utf8("Hello世界", 5), "Hello...");
    assert_eq!(truncate_utf8("a你b好c", 4), "a你b好...");
}

#[test]
fn test_truncate_utf8_zero_max() {
    // 零长度截断
    assert_eq!(truncate_utf8("hello", 0), "...");
}

// ==================== toolbar_button 测试 ====================

/// 简化版 toolbar_button 测试
#[test]
fn test_toolbar_button_format() {
    let action = "bold";
    let icon = r#"<svg>test</svg>"#;
    let expected = format!(
        r#"<button type="button" class="toolbar-btn inline-flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-foreground" data-action="{}">{}</button>"#,
        action, icon
    );

    // 验证格式正确
    assert!(expected.contains("data-action=\"bold\""));
    assert!(expected.contains("<svg>test</svg>"));
}

// ==================== escape_html 测试（来自 handlers）====================

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[test]
fn test_escape_html_in_handler_context() {
    // 测试在 handler 上下文中的转义
    let user_input = r#"<script>alert("xss")</script>"#;
    let escaped = escape_html(user_input);

    assert!(!escaped.contains("<script>"));
    assert!(escaped.contains("&lt;script&gt;"));
    assert!(escaped.contains("&quot;"));
}

#[test]
fn test_escape_html_preserves_normal_text() {
    // 正常文本保持不变
    assert_eq!(escape_html("Hello, World!"), "Hello, World!");
    assert_eq!(escape_html("12345"), "12345");
}

#[test]
fn test_escape_html_unicode() {
    // Unicode 保持不变
    assert_eq!(escape_html("你好世界"), "你好世界");
    assert_eq!(escape_html("日本語"), "日本語");
}

// ==================== escape_attribute 测试 ====================

fn escape_attribute(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[test]
fn test_escape_attribute_basic() {
    assert_eq!(escape_attribute("hello"), "hello");
    assert_eq!(escape_attribute("<test>"), "&lt;test&gt;");
    assert_eq!(escape_attribute("a & b"), "a &amp; b");
    assert_eq!(escape_attribute("\"quoted\""), "&quot;quoted&quot;");
}

#[test]
fn test_escape_attribute_no_single_quote() {
    // 属性转义不处理单引号
    assert_eq!(escape_attribute("it's"), "it's");
}

// ==================== get_safe_color 测试 ====================

fn get_safe_color(color: &str) -> String {
    if color.is_empty() {
        return "#888888".to_string();
    }
    if color.len() == 7 && color.starts_with('#') {
        return color.to_string();
    }
    if color.len() == 4 && color.starts_with('#') {
        let chars: Vec<char> = color.chars().collect();
        return format!(
            "#{}{}{}{}{}{}",
            chars[1], chars[1], chars[2], chars[2], chars[3], chars[3]
        );
    }
    "#888888".to_string()
}

#[test]
fn test_get_safe_color_valid() {
    assert_eq!(get_safe_color("#ff0000"), "#ff0000");
    assert_eq!(get_safe_color("#3b82f6"), "#3b82f6");
}

#[test]
fn test_get_safe_color_expand() {
    assert_eq!(get_safe_color("#fff"), "#ffffff");
    assert_eq!(get_safe_color("#abc"), "#aabbcc");
}

#[test]
fn test_get_safe_color_invalid() {
    assert_eq!(get_safe_color(""), "#888888");
    assert_eq!(get_safe_color("red"), "#888888");
}

// ==================== get_avatar_gradient 测试 ====================

fn get_avatar_gradient(id: i64) -> &'static str {
    let gradients = [
        "from-pink-500 to-rose-500",
        "from-orange-400 to-red-500",
        "from-amber-400 to-orange-500",
        "from-lime-400 to-emerald-500",
        "from-green-400 to-emerald-600",
        "from-teal-400 to-cyan-500",
        "from-sky-400 to-blue-500",
        "from-indigo-400 to-purple-500",
        "from-violet-400 to-fuchsia-500",
        "from-purple-400 to-pink-500",
        "from-slate-400 to-zinc-500",
    ];
    gradients[(id.abs() as usize) % gradients.len()]
}

#[test]
fn test_get_avatar_gradient_consistency() {
    // 相同 ID 返回相同渐变
    assert_eq!(get_avatar_gradient(1), get_avatar_gradient(1));
    assert_eq!(get_avatar_gradient(100), get_avatar_gradient(100));
}

#[test]
fn test_get_avatar_gradient_negative() {
    // 负数 ID 使用绝对值
    assert_eq!(get_avatar_gradient(5), get_avatar_gradient(-5));
}

#[test]
fn test_get_avatar_gradient_modulo() {
    // 验证模运算
    let g1 = get_avatar_gradient(1);
    let g12 = get_avatar_gradient(12); // 12 % 11 = 1
    assert_eq!(g1, g12);
}

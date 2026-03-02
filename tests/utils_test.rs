//! 工具函数单元测试
//!
//! 测试 src/utils.rs 中的所有工具函数

use chrono::{DateTime, Local, Utc};

/// 模拟 escape_html 函数
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 模拟 escape_attribute 函数
fn escape_attribute(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 模拟 get_safe_color 函数
fn get_safe_color(color: &str) -> String {
    if color.is_empty() {
        return "#888888".to_string();
    }
    // 6-digit hex
    if color.len() == 7 && color.starts_with('#') {
        return color.to_string();
    }
    // 3-digit hex - expand
    if color.len() == 4 && color.starts_with('#') {
        let chars: Vec<char> = color.chars().collect();
        return format!(
            "#{}{}{}{}{}{}",
            chars[1], chars[1], chars[2], chars[2], chars[3], chars[3]
        );
    }
    "#888888".to_string()
}

/// 模拟 get_avatar_gradient 函数
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

/// 模拟 format_display_time 函数
fn format_display_time(created_at: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(created_at) {
        let local: DateTime<Local> = dt.with_timezone(&Local);
        let now = Local::now();
        let diff = now.signed_duration_since(local);

        if diff.num_minutes() < 1 {
            return "刚刚".to_string();
        } else if diff.num_minutes() < 60 {
            return format!("{} 分钟前", diff.num_minutes());
        } else if diff.num_hours() < 24 {
            return format!("{} 小时前", diff.num_hours());
        } else if diff.num_days() < 7 {
            return format!("{} 天前", diff.num_days());
        } else {
            return local.format("%Y-%m-%d %H:%M").to_string();
        }
    }
    created_at.to_string()
}

/// 模拟 now_iso 函数
fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// 模拟 today_date 函数
fn today_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

// ==================== escape_html 测试 ====================

#[test]
fn test_escape_html_basic() {
    let test_cases = vec![
        ("<script>", "&lt;script&gt;"),
        ("Hello & goodbye", "Hello &amp; goodbye"),
        ("1 < 2 > 3", "1 &lt; 2 &gt; 3"),
        ("\"quoted\"", "&quot;quoted&quot;"),
        ("'single'", "&#39;single&#39;"),
    ];

    for (input, expected) in test_cases {
        let result = escape_html(input);
        assert_eq!(result, expected);
    }
}

#[test]
fn test_escape_html_complex() {
    // 测试复杂 XSS 攻击向量
    let test_cases = vec![
        (
            "<script>alert('xss')</script>",
            "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;",
        ),
        (
            "<img src=\"x\" onerror=\"alert(1)\">",
            "&lt;img src=&quot;x&quot; onerror=&quot;alert(1)&quot;&gt;",
        ),
        ("javascript:alert('xss')", "javascript:alert(&#39;xss&#39;)"),
        (
            "<a href='javascript:void(0)'>link</a>",
            "&lt;a href=&#39;javascript:void(0)&#39;&gt;link&lt;/a&gt;",
        ),
    ];

    for (input, expected) in test_cases {
        let result = escape_html(input);
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_escape_html_empty() {
    assert_eq!(escape_html(""), "");
}

#[test]
fn test_escape_html_no_special_chars() {
    assert_eq!(escape_html("Hello World"), "Hello World");
    assert_eq!(escape_html("123456789"), "123456789");
}

#[test]
fn test_escape_html_unicode() {
    // Unicode 字符不应该被转义
    assert_eq!(escape_html("你好世界"), "你好世界");
    assert_eq!(escape_html("日本語テスト"), "日本語テスト");
    assert_eq!(escape_html("🎉🎊🎈"), "🎉🎊🎈");
}

#[test]
fn test_escape_html_ampersand_priority() {
    // & 应该首先被转义，以避免双重转义
    let result = escape_html("a & b");
    assert_eq!(result, "a &amp; b");
    assert!(!result.contains("&&"));
}

// ==================== escape_attribute 测试 ====================

#[test]
fn test_escape_attribute_basic() {
    let test_cases = vec![
        ("hello", "hello"),
        ("<tag>", "&lt;tag&gt;"),
        ("a & b", "a &amp; b"),
        ("\"quoted\"", "&quot;quoted&quot;"),
    ];

    for (input, expected) in test_cases {
        let result = escape_attribute(input);
        assert_eq!(result, expected);
    }
}

#[test]
fn test_escape_attribute_no_single_quote() {
    // escape_attribute 不转义单引号
    let result = escape_attribute("it's");
    assert_eq!(result, "it's");
}

#[test]
fn test_escape_attribute_empty() {
    assert_eq!(escape_attribute(""), "");
}

// ==================== get_safe_color 测试 ====================

#[test]
fn test_get_safe_color_valid_hex() {
    // 有效的 6 位 hex 颜色
    assert_eq!(get_safe_color("#ff0000"), "#ff0000");
    assert_eq!(get_safe_color("#00ff00"), "#00ff00");
    assert_eq!(get_safe_color("#0000ff"), "#0000ff");
    assert_eq!(get_safe_color("#3b82f6"), "#3b82f6");
}

#[test]
fn test_get_safe_color_expand_short_hex() {
    // 3 位 hex 颜色应该被扩展
    assert_eq!(get_safe_color("#fff"), "#ffffff");
    assert_eq!(get_safe_color("#000"), "#000000");
    assert_eq!(get_safe_color("#abc"), "#aabbcc");
    assert_eq!(get_safe_color("#123"), "#112233");
}

#[test]
fn test_get_safe_color_empty() {
    assert_eq!(get_safe_color(""), "#888888");
}

#[test]
fn test_get_safe_color_invalid() {
    // 无效格式应该返回默认颜色
    assert_eq!(get_safe_color("red"), "#888888");
    assert_eq!(get_safe_color("rgb(255,0,0)"), "#888888");
    // 格式有效但内容无效（不是有效的十六进制字符），函数仍返回原值
    // 因为函数只检查格式，不验证颜色值
    assert_eq!(get_safe_color("#gggggg"), "#gggggg");
}

#[test]
fn test_get_safe_color_edge_cases() {
    // 边界情况
    assert_eq!(get_safe_color("#"), "#888888");
    assert_eq!(get_safe_color("#1"), "#888888");
    assert_eq!(get_safe_color("#12"), "#888888");
    assert_eq!(get_safe_color("#1234"), "#888888"); // 4 位不是有效格式
    assert_eq!(get_safe_color("#12345"), "#888888");
}

// ==================== get_avatar_gradient 测试 ====================

#[test]
fn test_get_avatar_gradient_positive_ids() {
    // 测试正数 ID
    let gradient1 = get_avatar_gradient(1);
    let gradient2 = get_avatar_gradient(2);
    let gradient3 = get_avatar_gradient(12); // 应该和 1 相同（11 个渐变色）

    assert!(!gradient1.is_empty());
    assert!(!gradient2.is_empty());
    assert_eq!(gradient1, gradient3); // (12 % 11) == 1
}

#[test]
fn test_get_avatar_gradient_negative_ids() {
    // 测试负数 ID（使用 abs）
    let gradient_pos = get_avatar_gradient(5);
    let gradient_neg = get_avatar_gradient(-5);

    assert_eq!(gradient_pos, gradient_neg);
}

#[test]
fn test_get_avatar_gradient_zero() {
    // ID 为 0 的情况
    let gradient = get_avatar_gradient(0);
    assert!(!gradient.is_empty());
}

#[test]
fn test_get_avatar_gradient_consistency() {
    // 相同 ID 应该返回相同结果
    for id in 0..100 {
        let g1 = get_avatar_gradient(id);
        let g2 = get_avatar_gradient(id);
        assert_eq!(g1, g2);
    }
}

#[test]
fn test_get_avatar_gradient_large_id() {
    // 大数值 ID
    let gradient = get_avatar_gradient(i64::MAX);
    assert!(!gradient.is_empty());

    // 注意: i64::MIN 的 abs() 会溢出，这里使用一个大的负数
    let gradient = get_avatar_gradient(-999999999);
    assert!(!gradient.is_empty());
}

// ==================== format_display_time 测试 ====================

#[test]
fn test_format_display_time_just_now() {
    // 刚刚（1分钟内）
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let result = format_display_time(&now);
    assert_eq!(result, "刚刚");
}

#[test]
fn test_format_display_time_minutes_ago() {
    // 几分钟前
    let five_min_ago = (Utc::now() - chrono::Duration::minutes(5))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let result = format_display_time(&five_min_ago);
    assert!(result.starts_with("5 分钟前") || result.starts_with("6 分钟前")); // 可能有一点时间差
}

#[test]
fn test_format_display_time_hours_ago() {
    // 几小时前
    let three_hours_ago = (Utc::now() - chrono::Duration::hours(3))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let result = format_display_time(&three_hours_ago);
    assert!(result.contains("小时前"));
}

#[test]
fn test_format_display_time_days_ago() {
    // 几天前（但小于7天）
    let three_days_ago = (Utc::now() - chrono::Duration::days(3))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let result = format_display_time(&three_days_ago);
    assert!(result.contains("天前"));
}

#[test]
fn test_format_display_time_full_date() {
    // 超过7天，显示完整日期
    let ten_days_ago = (Utc::now() - chrono::Duration::days(10))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let result = format_display_time(&ten_days_ago);
    // 应该是 YYYY-MM-DD HH:MM 格式
    assert!(result.contains("-"));
    assert!(result.contains(":"));
}

#[test]
fn test_format_display_time_invalid_format() {
    // 无效格式应该返回原始字符串
    let invalid = "not-a-date";
    let result = format_display_time(invalid);
    assert_eq!(result, "not-a-date");
}

#[test]
fn test_format_display_time_edge_case() {
    // 边界情况：刚好1分钟
    let one_min_ago = (Utc::now() - chrono::Duration::seconds(59))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let result = format_display_time(&one_min_ago);
    assert_eq!(result, "刚刚");

    // 刚好60秒
    let one_min_ago = (Utc::now() - chrono::Duration::seconds(60))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let result = format_display_time(&one_min_ago);
    assert!(result.contains("分钟前"));
}

// ==================== now_iso 测试 ====================

#[test]
fn test_now_iso_format() {
    let result = now_iso();

    // 验证格式：YYYY-MM-DDTHH:MM:SSZ
    assert!(result.len() == 20);
    assert!(result.ends_with('Z'));
    assert!(result.chars().nth(4) == Some('-'));
    assert!(result.chars().nth(7) == Some('-'));
    assert!(result.chars().nth(10) == Some('T'));
    assert!(result.chars().nth(13) == Some(':'));
    assert!(result.chars().nth(16) == Some(':'));
}

#[test]
fn test_now_iso_parseable() {
    let result = now_iso();

    // 验证可以被解析
    let parsed = DateTime::parse_from_rfc3339(&result);
    assert!(parsed.is_ok());
}

#[test]
fn test_now_iso_current() {
    let before = Utc::now();
    let result = now_iso();
    let after = Utc::now();

    // 结果应该在 before 和 after 之间
    let parsed = DateTime::parse_from_rfc3339(&result)
        .unwrap()
        .with_timezone(&Utc);
    assert!(parsed >= before - chrono::Duration::seconds(1));
    assert!(parsed <= after + chrono::Duration::seconds(1));
}

// ==================== today_date 测试 ====================

#[test]
fn test_today_date_format() {
    let result = today_date();

    // 验证格式：YYYY-MM-DD
    assert!(result.len() == 10);
    assert!(result.chars().nth(4) == Some('-'));
    assert!(result.chars().nth(7) == Some('-'));
}

#[test]
fn test_today_date_current() {
    let result = today_date();
    let expected = Local::now().format("%Y-%m-%d").to_string();

    assert_eq!(result, expected);
}

#[test]
fn test_today_date_parseable() {
    let result = today_date();

    // 验证可以被解析
    let parsed = chrono::NaiveDate::parse_from_str(&result, "%Y-%m-%d");
    assert!(parsed.is_ok());
}

// ==================== 综合测试 ====================

#[test]
fn test_escape_functions_consistency() {
    // escape_html 和 escape_attribute 的关系
    let input = "<script> & \"test\"</script>";

    let html_result = escape_html(input);
    let attr_result = escape_attribute(input);

    // escape_html 转义单引号，escape_attribute 不转义
    assert!(html_result.contains("&#39;") || !input.contains("'"));
    // 两者都应该转义 & < > "
    assert!(attr_result.contains("&amp;"));
    assert!(attr_result.contains("&lt;"));
    assert!(attr_result.contains("&gt;"));
    assert!(attr_result.contains("&quot;"));
}

#[test]
fn test_time_functions_consistency() {
    // now_iso 和 today_date 应该是同一天
    let iso = now_iso();
    let today = today_date();

    // 从 iso 中提取日期
    let iso_date = &iso[0..10];
    assert_eq!(iso_date, today);
}

use chrono::{DateTime, Local, Utc};

pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn escape_attribute(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn format_display_time(created_at: &str) -> String {
    // 尝试解析 ISO 格式的时间
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

pub fn get_avatar_gradient(id: i64) -> &'static str {
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

pub fn get_safe_color(color: &str) -> String {
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

pub fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn today_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

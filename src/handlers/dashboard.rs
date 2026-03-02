use crate::config::VERSION;
use crate::db::Repository;
use crate::utils::*;
use actix_web::{web, HttpResponse};

struct TagRankingItem {
    name: String,
    color: String,
    usage_count: i64,
    percentage: f64,
}

struct TopMessageItem {
    content: String,
    reply_count: i64,
}

/// UTF-8 安全的字符串截断
fn truncate_utf8(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect::<String>() + "..."
    }
}

pub async fn dashboard(repo: web::Data<Repository>) -> HttpResponse {
    // 获取统计数据
    let total_messages_ever = repo.get_stat("total_messages_ever").await.unwrap_or(0);
    let current_message_count = repo.count_messages().await.unwrap_or(0);
    let current_reply_count = repo.get_total_replies().await.unwrap_or(0);

    // 获取真实的平均留言长度
    let avg_message_length = repo.get_average_message_length().await.unwrap_or(0.0) as i64;

    // 获取标签排行
    let tags = repo.get_tags_with_count().await.unwrap_or_default();
    let max_count = tags.iter().map(|t| t.count).max().unwrap_or(1);
    let tag_ranking: Vec<TagRankingItem> = tags
        .iter()
        .filter(|t| t.count > 0)
        .take(10)
        .map(|t| TagRankingItem {
            name: t.name.clone(),
            color: get_safe_color(&t.color),
            usage_count: t.count,
            percentage: (t.count as f64 / max_count as f64) * 100.0,
        })
        .collect();

    // 获取每日统计
    let daily_stats = repo.get_daily_stats().await.unwrap_or_default();
    let daily_labels: Vec<String> = daily_stats
        .iter()
        .map(|d| {
            // UTF-8 安全截取：从第6个字符开始（跳过 "YYYY-"）
            if d.date.len() > 5 {
                d.date[5..].to_string()
            } else {
                d.date.clone()
            }
        })
        .collect();
    let daily_message_data: Vec<i64> = daily_stats.iter().map(|d| d.message_count).collect();
    let daily_reply_data: Vec<i64> = daily_stats.iter().map(|d| d.reply_count).collect();

    // 获取真实的时段分布数据
    let hourly_data = repo
        .get_hourly_distribution()
        .await
        .unwrap_or_else(|_| vec![0; 24]);

    // 获取真实的热门留言
    let top_msg_data = repo
        .get_top_messages_by_replies(5)
        .await
        .unwrap_or_default();
    let top_messages: Vec<TopMessageItem> = top_msg_data
        .into_iter()
        .map(|(content, reply_count)| TopMessageItem {
            content: truncate_utf8(&content, 80),
            reply_count,
        })
        .collect();

    let html = render_dashboard(
        total_messages_ever,
        current_message_count,
        current_reply_count,
        avg_message_length,
        &tag_ranking,
        &daily_labels,
        &daily_message_data,
        &daily_reply_data,
        &hourly_data,
        &top_messages,
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn render_dashboard(
    total_messages_ever: i64,
    current_message_count: i64,
    current_reply_count: i64,
    avg_message_length: i64,
    tag_ranking: &[TagRankingItem],
    daily_labels: &[String],
    daily_message_data: &[i64],
    daily_reply_data: &[i64],
    hourly_data: &[i64],
    top_messages: &[TopMessageItem],
) -> String {
    let tag_ranking_html = render_tag_ranking(tag_ranking);
    let top_messages_html = render_top_messages(top_messages);

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>数据看板 - 简易留言板</title>
    <script>
        (function() {{
            try {{
                const storedTheme = localStorage.getItem('theme');
                if (storedTheme === 'dark') {{
                    document.documentElement.classList.add('dark');
                }} else if (storedTheme === 'light') {{
                }} else {{
                    document.documentElement.classList.add('dark', 'cyberpunk');
                }}
            }} catch (error) {{
                document.documentElement.classList.add('dark', 'cyberpunk');
            }}
        }})();
    </script>
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    colors: {{
                        border: 'hsl(var(--border))',
                        input: 'hsl(var(--input))',
                        ring: 'hsl(var(--ring))',
                        background: 'hsl(var(--background))',
                        foreground: 'hsl(var(--foreground))',
                        primary: {{ DEFAULT: 'hsl(var(--primary))', foreground: 'hsl(var(--primary-foreground))' }},
                        secondary: {{ DEFAULT: 'hsl(var(--secondary))', foreground: 'hsl(var(--secondary-foreground))' }},
                        destructive: {{ DEFAULT: 'hsl(var(--destructive))', foreground: 'hsl(var(--destructive-foreground))' }},
                        muted: {{ DEFAULT: 'hsl(var(--muted))', foreground: 'hsl(var(--muted-foreground))' }},
                        accent: {{ DEFAULT: 'hsl(var(--accent))', foreground: 'hsl(var(--accent-foreground))' }},
                        card: {{ DEFAULT: 'hsl(var(--card))', foreground: 'hsl(var(--card-foreground))' }}
                    }},
                    borderRadius: {{ lg: 'var(--radius)', md: 'calc(var(--radius) - 2px)', sm: 'calc(var(--radius) - 4px)' }},
                    fontFamily: {{ sans: ['Inter', 'system-ui', 'sans-serif'], mono: ['JetBrains Mono', 'monospace'] }}
                }}
            }}
        }};
    </script>
    <style>
        :root {{
            color-scheme: light;
            --background: 0 0% 100%; --foreground: 222.2 47.4% 11.2%;
            --muted: 210 40% 96.1%; --muted-foreground: 215.4 16.3% 46.9%;
            --border: 214.3 31.8% 91.4%; --input: 214.3 31.8% 91.4%;
            --card: 0 0% 100%; --card-foreground: 222.2 47.4% 11.2%;
            --primary: 221.2 83.2% 53.3%; --primary-foreground: 210 40% 98%;
            --secondary: 210 40% 96.1%; --secondary-foreground: 222.2 47.4% 11.2%;
            --accent: 210 40% 96.1%; --accent-foreground: 222.2 47.4% 11.2%;
            --destructive: 0 72.2% 50.6%; --destructive-foreground: 210 40% 98%;
            --ring: 221.2 83.2% 53.3%; --radius: 0.9rem;
        }}
        .dark {{
            color-scheme: dark;
            --background: 222.2 84% 4.9%; --foreground: 210 40% 98%;
            --muted: 217.2 32.6% 17.5%; --muted-foreground: 215 20.2% 65.1%;
            --border: 217.2 32.6% 17.5%; --input: 217.2 32.6% 17.5%;
            --card: 222.2 84% 4.9%; --card-foreground: 210 40% 98%;
            --primary: 217.2 91.2% 59.8%; --primary-foreground: 222.2 47.4% 11.2%;
            --secondary: 217.2 32.6% 17.5%; --secondary-foreground: 210 40% 98%;
            --accent: 217.2 32.6% 17.5%; --accent-foreground: 210 40% 98%;
            --destructive: 0 62.8% 45.6%; --destructive-foreground: 210 40% 98%;
            --ring: 224.3 76.3% 48%;
        }}
    </style>
    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family=inter:400,500,600|jetbrains-mono:400,500" rel="stylesheet" />
    <link rel="stylesheet" href="/static/themes/theme-cyberpunk.css" id="theme-stylesheet">
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body class="min-h-screen bg-background text-foreground">
    <div class="relative isolate">
        <div class="pointer-events-none absolute inset-x-0 top-[-14rem] -z-10 transform-gpu overflow-hidden blur-3xl" aria-hidden="true">
            <div class="relative left-1/2 aspect-[1108/632] w-[72rem] -translate-x-1/2 bg-gradient-to-tr from-indigo-300 via-sky-200 to-purple-200 opacity-60 dark:from-indigo-950 dark:via-slate-800 dark:to-purple-900"></div>
        </div>
        <main class="mx-auto w-full max-w-6xl px-4 py-10 sm:px-6 lg:px-8">
            <div class="flex flex-col gap-6">
                <section class="flex flex-col gap-4 rounded-2xl border border-border bg-card/90 p-6 shadow-lg shadow-black/5 backdrop-blur">
                    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                        <div class="space-y-2">
                            <p class="text-xs uppercase tracking-[0.2em] text-muted-foreground">Analytics Dashboard</p>
                            <h1 class="text-3xl font-semibold tracking-tight">
                                <span>数据看板</span>
                                <span class="ml-2 text-base font-normal text-muted-foreground/60">v{}</span>
                            </h1>
                            <p class="text-sm text-muted-foreground">留言板运营数据统计与分析</p>
                        </div>
                        <div class="flex items-center gap-3 self-end sm:self-auto">
                            <a href="/" class="inline-flex h-9 items-center gap-2 rounded-md border border-input bg-background px-4 text-xs font-medium shadow-sm transition hover:bg-accent hover:text-accent-foreground">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/></svg>
                                <span>返回首页</span>
                            </a>
                            <button type="button" id="theme-toggle" class="inline-flex h-9 items-center gap-2 rounded-md border border-input bg-background px-3 text-xs font-medium shadow-sm transition hover:bg-accent hover:text-accent-foreground">
                                <span aria-hidden="true">☀️</span>
                                <span class="theme-toggle-label">亮色</span>
                            </button>
                        </div>
                    </div>
                </section>

                <section class="grid grid-cols-2 gap-4 lg:grid-cols-4">
                    {}
                    {}
                    {}
                    {}
                </section>

                <section class="grid gap-6 lg:grid-cols-2">
                    <div class="rounded-xl border border-border bg-card/90 p-5 shadow-sm backdrop-blur">
                        <h3 class="text-sm font-semibold mb-4 flex items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><line x1="12" x2="12" y1="20" y2="10"/><line x1="18" x2="18" y1="20" y2="4"/><line x1="6" x2="6" y1="20" y2="16"/></svg>
                            <span>每日留言趋势（最近30天）</span>
                        </h3>
                        <div class="h-64">
                            <canvas id="dailyChart"></canvas>
                        </div>
                    </div>
                    <div class="rounded-xl border border-border bg-card/90 p-5 shadow-sm backdrop-blur">
                        <h3 class="text-sm font-semibold mb-4 flex items-center gap-2">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                            <span>活跃时段分布（UTC 24小时）</span>
                        </h3>
                        <div class="h-64">
                            <canvas id="hourlyChart"></canvas>
                        </div>
                    </div>
                </section>

                {}
                {}
            </div>
        </main>
    </div>
    <script>
        const dailyLabels = {};
        const dailyMessageData = {};
        const dailyReplyData = {};
        const hourlyData = {};

        function getChartColors() {{
            const isDark = document.documentElement.classList.contains('dark');
            return {{
                text: isDark ? 'rgba(255,255,255,0.7)' : 'rgba(0,0,0,0.7)',
                grid: isDark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)',
                primary: isDark ? 'rgba(96, 165, 250, 0.8)' : 'rgba(59, 130, 246, 0.8)',
                primaryBg: isDark ? 'rgba(96, 165, 250, 0.2)' : 'rgba(59, 130, 246, 0.2)',
                secondary: isDark ? 'rgba(251, 191, 36, 0.8)' : 'rgba(245, 158, 11, 0.8)',
                secondaryBg: isDark ? 'rgba(251, 191, 36, 0.2)' : 'rgba(245, 158, 11, 0.2)'
            }};
        }}

        const colors = getChartColors();

        const dailyCtx = document.getElementById('dailyChart')?.getContext('2d');
        if (dailyCtx) {{
            new Chart(dailyCtx, {{
                type: 'line',
                data: {{
                    labels: dailyLabels,
                    datasets: [{{
                        label: '留言数',
                        data: dailyMessageData,
                        borderColor: colors.primary,
                        backgroundColor: colors.primaryBg,
                        fill: true,
                        tension: 0.4
                    }}, {{
                        label: '答复数',
                        data: dailyReplyData,
                        borderColor: colors.secondary,
                        backgroundColor: colors.secondaryBg,
                        fill: true,
                        tension: 0.4
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {{ legend: {{ labels: {{ color: colors.text }} }} }},
                    scales: {{
                        x: {{ ticks: {{ color: colors.text }}, grid: {{ color: colors.grid }} }},
                        y: {{ ticks: {{ color: colors.text }}, grid: {{ color: colors.grid }}, beginAtZero: true }}
                    }}
                }}
            }});
        }}

        const hourlyCtx = document.getElementById('hourlyChart')?.getContext('2d');
        if (hourlyCtx) {{
            const hourLabels = Array.from({{length: 24}}, (_, i) => i + ':00');
            new Chart(hourlyCtx, {{
                type: 'bar',
                data: {{
                    labels: hourLabels,
                    datasets: [{{
                        label: '留言数',
                        data: hourlyData,
                        backgroundColor: colors.primary,
                        borderRadius: 4
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {{ legend: {{ display: false }} }},
                    scales: {{
                        x: {{ ticks: {{ color: colors.text, maxRotation: 45 }}, grid: {{ display: false }} }},
                        y: {{ ticks: {{ color: colors.text }}, grid: {{ color: colors.grid }}, beginAtZero: true }}
                    }}
                }}
            }});
        }}

        const themeToggle = document.getElementById('theme-toggle');
        const root = document.documentElement;
        function updateThemeToggle() {{
            const isDark = root.classList.contains('dark');
            const icon = themeToggle?.querySelector('span[aria-hidden="true"]');
            const label = themeToggle?.querySelector('.theme-toggle-label');
            if (icon) icon.textContent = isDark ? '🌙' : '☀️';
            if (label) label.textContent = isDark ? '暗色' : '亮色';
        }}
        themeToggle?.addEventListener('click', () => {{
            const isDark = root.classList.toggle('dark');
            try {{ localStorage.setItem('theme', isDark ? 'dark' : 'light'); }} catch(e) {{}}
            updateThemeToggle();
            location.reload();
        }});
        updateThemeToggle();
    </script>
</body>
</html>"#,
        VERSION,
        stat_card(
            total_messages_ever,
            "历史总留言",
            "text-primary",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>"#
        ),
        stat_card(
            current_message_count,
            "当前留言数",
            "text-emerald-500",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>"#
        ),
        stat_card(
            current_reply_count,
            "总答复数",
            "text-amber-500",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 21 1.9-5.7a8.5 8.5 0 1 1 3.8 3.8z"/></svg>"#
        ),
        stat_card(
            avg_message_length,
            "平均字数",
            "text-violet-500",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/></svg>"#
        ),
        tag_ranking_html,
        top_messages_html,
        serde_json::to_string(daily_labels).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string(daily_message_data).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string(daily_reply_data).unwrap_or_else(|_| "[]".to_string()),
        serde_json::to_string(hourly_data).unwrap_or_else(|_| "[]".to_string()),
    )
}

fn stat_card(value: i64, label: &str, color_class: &str, icon: &str) -> String {
    format!(
        r#"<div class="rounded-xl border border-border bg-card/90 p-5 shadow-sm backdrop-blur transition hover:shadow-md">
        <div class="flex items-center justify-between">
            <div class="rounded-lg bg-muted p-2 {}">{}</div>
        </div>
        <div class="mt-4">
            <p class="text-2xl font-bold tracking-tight">{}</p>
            <p class="text-xs text-muted-foreground mt-1">{}</p>
        </div>
    </div>"#,
        color_class, icon, value, label
    )
}

fn render_tag_ranking(tags: &[TagRankingItem]) -> String {
    if tags.is_empty() {
        return r#"<section class="rounded-xl border border-border bg-card/90 p-5 shadow-sm backdrop-blur">
            <h3 class="text-sm font-semibold mb-4 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><path d="M12 2H2v10l9.29 9.29c.94.94 2.48.94 3.42 0l6.58-6.58c.94-.94.94-2.48 0-3.42L12 2Z"/><path d="M7 7h.01"/></svg>
                <span>标签使用排行</span>
            </h3>
            <p class="text-sm text-muted-foreground">暂无标签数据</p>
        </section>"#.to_string();
    }

    let items: Vec<String> = tags.iter().enumerate().map(|(i, tag)| {
        format!(r#"<div class="flex items-center gap-3">
                <span class="w-6 text-xs text-muted-foreground text-right">{}</span>
                <div class="flex-1">
                    <div class="flex items-center justify-between mb-1">
                        <span class="text-sm font-medium flex items-center gap-1.5">
                            <span class="inline-block h-2 w-2 rounded-full" style="background-color: {};"></span>
                            {}
                        </span>
                        <span class="text-xs text-muted-foreground">{} 次</span>
                    </div>
                    <div class="h-2 rounded-full bg-muted overflow-hidden">
                        <div class="h-full rounded-full transition-all" style="width: {:.0}%; background-color: {};"></div>
                    </div>
                </div>
            </div>"#,
            i + 1,
            tag.color,
            escape_html(&tag.name),
            tag.usage_count,
            tag.percentage,
            tag.color
        )
    }).collect();

    format!(
        r#"<section class="rounded-xl border border-border bg-card/90 p-5 shadow-sm backdrop-blur">
            <h3 class="text-sm font-semibold mb-4 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><path d="M12 2H2v10l9.29 9.29c.94.94 2.48.94 3.42 0l6.58-6.58c.94-.94.94-2.48 0-3.42L12 2Z"/><path d="M7 7h.01"/></svg>
                <span>标签使用排行 TOP {}</span>
            </h3>
            <div class="space-y-3">
                {}
            </div>
        </section>"#,
        tags.len(),
        items.join("")
    )
}

fn render_top_messages(messages: &[TopMessageItem]) -> String {
    if messages.is_empty() {
        return String::new();
    }

    // 内容已在调用前被安全截断
    let items: Vec<String> = messages.iter().enumerate().map(|(i, msg)| {
        format!(r#"<div class="flex items-start gap-3 p-3 rounded-lg bg-muted/30 hover:bg-muted/50 transition">
                <span class="flex-shrink-0 w-6 h-6 rounded-full bg-primary/10 text-primary text-xs font-bold flex items-center justify-center">{}</span>
                <div class="flex-1 min-w-0">
                    <p class="text-sm text-foreground line-clamp-2">{}</p>
                    <p class="text-xs text-muted-foreground mt-1">{} 条答复</p>
                </div>
            </div>"#,
            i + 1,
            escape_html(&msg.content),
            msg.reply_count
        )
    }).collect();

    format!(
        r#"<section class="rounded-xl border border-border bg-card/90 p-5 shadow-sm backdrop-blur">
            <h3 class="text-sm font-semibold mb-4 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"/></svg>
                <span>热门留言 TOP {}</span>
            </h3>
            <div class="space-y-2">
                {}
            </div>
        </section>"#,
        messages.len(),
        items.join("")
    )
}

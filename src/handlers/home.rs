use crate::config::{
    MAX_MESSAGES, MAX_PAGES, MAX_SEARCH_LENGTH, PAGE_SIZE_OPTIONS, VERSION, VERSION_DATE,
};
use crate::db::Repository;
use crate::utils::*;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParams {
    q: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
    tag: Option<String>,
}

struct MessageView {
    id: i64,
    content: String,
    created_at: String,
    display_time: String,
    avatar_gradient: String,
    tags: Vec<TagView>,
    replies: Vec<ReplyView>,
}

struct TagView {
    id: i64,
    name: String,
    color: String,
}

struct ReplyView {
    id: i64,
    content: String,
    created_at: String,
    display_time: String,
    avatar_gradient: String,
}

struct TagSidebarItem {
    id: i64,
    name: String,
    color: String,
    message_count: i64,
}

pub async fn home(repo: web::Data<Repository>, query: web::Query<QueryParams>) -> HttpResponse {
    // 限制搜索词长度，防止DoS
    let search_term: String = query
        .q
        .as_deref()
        .map(|s| s.chars().take(MAX_SEARCH_LENGTH).collect())
        .unwrap_or_default();
    let page_size = normalize_page_size(query.page_size);
    let max_pages = ((MAX_MESSAGES + page_size - 1) / page_size).max(1);
    // 限制页码范围，防止过大的页码请求
    let current_page = query.page.unwrap_or(1).clamp(1, max_pages.min(MAX_PAGES));
    let tag_filter = query.tag.clone().unwrap_or_default();

    // 验证 tag_filter 是否为有效的数字ID（防止注入）
    let tag_id_opt: Option<i64> = if !tag_filter.is_empty() {
        tag_filter.parse().ok().filter(|&id| id > 0)
    } else {
        None
    };

    // 获取留言（使用批量查询避免N+1）
    let messages = if !search_term.is_empty() {
        repo.search_messages_with_tags_batch(&search_term, current_page, page_size)
            .await
    } else if let Some(tag_id) = tag_id_opt {
        repo.get_messages_by_tag_with_tags_batch(tag_id, current_page, page_size)
            .await
    } else {
        repo.get_messages_with_tags_batch(current_page, page_size)
            .await
    };

    let messages = match messages {
        Ok(m) => m,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    // 获取总数
    let total_messages = if !search_term.is_empty() {
        repo.count_search_messages(&search_term).await.unwrap_or(0)
    } else if let Some(tag_id) = tag_id_opt {
        repo.count_messages_by_tag(tag_id).await.unwrap_or(0)
    } else {
        repo.count_messages().await.unwrap_or(0)
    };

    // 历史总留言数：优先从 stats 表读取 total_messages_ever，如果没有则使用当前留言数
    let total_messages_ever = repo.get_stat("total_messages_ever").await.unwrap_or(0);
    let total_messages_ever = std::cmp::max(total_messages_ever, total_messages);

    // 计算分页，限制最大页数
    let total_pages = ((total_messages + page_size - 1) / page_size).max(1);
    let total_pages = total_pages.min(max_pages.min(MAX_PAGES));

    // 获取所有标签
    let all_tags = repo.get_tags_with_count().await.unwrap_or_default();
    let tag_sidebar: Vec<TagSidebarItem> = all_tags
        .iter()
        .map(|t| TagSidebarItem {
            id: t.id,
            name: t.name.clone(),
            color: get_safe_color(&t.color),
            message_count: t.count,
        })
        .collect();

    // 批量获取所有留言的回复（避免N+1）
    let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
    let all_replies = repo
        .get_replies_for_messages_batch(&message_ids)
        .await
        .unwrap_or_default();

    let message_views: Vec<MessageView> = messages
        .into_iter()
        .map(|msg| {
            let reply_views: Vec<ReplyView> = all_replies
                .get(&msg.id)
                .map(|replies| {
                    replies
                        .iter()
                        .map(|r| ReplyView {
                            id: r.id,
                            content: r.content.clone(),
                            created_at: r.created_at.clone(),
                            display_time: format_display_time(&r.created_at),
                            avatar_gradient: get_avatar_gradient(r.id).to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default();

            let tag_views: Vec<TagView> = msg
                .tags
                .iter()
                .map(|t| TagView {
                    id: t.id,
                    name: t.name.clone(),
                    color: get_safe_color(&t.color),
                })
                .collect();

            MessageView {
                id: msg.id,
                content: msg.content.clone(),
                created_at: msg.created_at.clone(),
                display_time: format_display_time(&msg.created_at),
                avatar_gradient: get_avatar_gradient(msg.id).to_string(),
                tags: tag_views,
                replies: reply_views,
            }
        })
        .collect();

    // 生成页面数字
    let pages = generate_pages(current_page, total_pages);

    let html = render_home_page(
        message_views,
        tag_sidebar,
        search_term,
        current_page,
        page_size,
        total_pages,
        total_messages,
        total_messages_ever,
        tag_filter,
        pages,
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

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

#[allow(clippy::too_many_arguments)]
fn render_home_page(
    messages: Vec<MessageView>,
    all_tags: Vec<TagSidebarItem>,
    search_term: String,
    current_page: i64,
    page_size: i64,
    total_pages: i64,
    total_messages: i64,
    total_messages_ever: i64,
    tag_filter: String,
    pages: Vec<String>,
) -> String {
    let prev_page = (current_page - 1).max(1);
    let next_page = (current_page + 1).min(total_pages);

    let tag_sidebar_html = render_tag_sidebar(&all_tags, &tag_filter, page_size);
    let messages_html = render_messages(&messages, current_page, page_size, &search_term, &tag_filter);
    let pagination_html = if total_pages > 1 {
        render_pagination(
            current_page,
            page_size,
            total_pages,
            &search_term,
            &tag_filter,
            &pages,
            prev_page,
            next_page,
        )
    } else {
        String::new()
    };

    let search_display = if !search_term.is_empty() {
        format!(
            r#"<span class="inline-flex items-center gap-1 rounded-full bg-primary/10 px-2.5 py-0.5 text-xs font-medium text-primary">
            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/></svg>
            <span>已筛选：{}</span>
        </span>"#,
            escape_html(&search_term)
        )
    } else {
        String::new()
    };

    let clear_search = if !search_term.is_empty() {
        format!(
            r#"<a href="/?page_size={}" class="inline-flex h-10 items-center justify-center rounded-md border border-input bg-background px-4 text-sm font-medium transition hover:bg-accent hover:text-accent-foreground">清除</a>"#,
            page_size
        )
    } else {
        String::new()
    };

    let page_size_selector = render_page_size_selector(page_size);

    let toolbar = format!(
        r#"{}{}{}{}{}{}{}{}{}{}{}{}"#,
        toolbar_button(
            "heading-1",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 12h8"/><path d="M4 18V6"/><path d="M12 18V6"/><path d="m17 12 3-2v8"/></svg>"#
        ),
        toolbar_button(
            "heading-2",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 12h8"/><path d="M4 18V6"/><path d="M12 18V6"/><path d="M21 18h-4c0-4 4-3 4-6 0-1.5-2-2.5-4-1"/></svg>"#
        ),
        toolbar_divider(),
        toolbar_button(
            "bold",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 12a4 4 0 0 0 0-8H6v8"/><path d="M15 20a4 4 0 0 0 0-8H6v8Z"/></svg>"#
        ),
        toolbar_button(
            "italic",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="19" x2="10" y1="4" y2="4"/><line x1="14" x2="5" y1="20" y2="20"/><line x1="15" x2="9" y1="4" y2="20"/></svg>"#
        ),
        toolbar_button(
            "quote",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"/><path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V20c0 1 0 1 1 1z"/></svg>"#
        ),
        toolbar_divider(),
        toolbar_button(
            "list-ul",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" x2="21" y1="6" y2="6"/><line x1="8" x2="21" y1="12" y2="12"/><line x1="8" x2="21" y1="18" y2="18"/><line x1="3" x2="3.01" y1="6" y2="6"/><line x1="3" x2="3.01" y1="12" y2="12"/><line x1="3" x2="3.01" y1="18" y2="18"/></svg>"#
        ),
        toolbar_button(
            "list-ol",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="10" x2="21" y1="6" y2="6"/><line x1="10" x2="21" y1="12" y2="12"/><line x1="10" x2="21" y1="18" y2="18"/><path d="M4 6h1v4"/><path d="M4 10h2"/><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"/></svg>"#
        ),
        toolbar_divider(),
        toolbar_button(
            "code",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>"#
        ),
        toolbar_button(
            "code-block",
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 18H3"/><path d="M10 6H3"/><path d="M21 12H3"/><path d="m18 6 3 6-3 6"/></svg>"#
        ),
    );

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>简易留言板</title>
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
    <link rel="preconnect" href="https://fonts.bunny.net">
    <link href="https://fonts.bunny.net/css?family=inter:400,500,600|jetbrains-mono:400,500" rel="stylesheet" />
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css" referrerpolicy="no-referrer" />
    <link rel="stylesheet" href="/static/themes/theme-cyberpunk.css" id="theme-stylesheet">
    <link rel="stylesheet" href="/static/app.css">
</head>
<body class="min-h-screen bg-background text-foreground" data-search-term="{}" data-page-size="{}" data-tag-filter="{}">
    {}
    <div class="relative isolate">
        <div class="pointer-events-none absolute inset-x-0 top-[-14rem] -z-10 transform-gpu overflow-hidden blur-3xl" aria-hidden="true">
            <div class="relative left-1/2 aspect-[1108/632] w-[72rem] -translate-x-1/2 bg-gradient-to-tr from-indigo-300 via-sky-200 to-purple-200 opacity-60 dark:from-indigo-950 dark:via-slate-800 dark:to-purple-900"></div>
        </div>
        <main class="mx-auto w-full max-w-5xl px-4 py-10 sm:px-6 lg:px-8">
            <div class="flex flex-col gap-6">
                <section class="flex flex-col gap-4 rounded-2xl border border-border bg-card/90 p-6 shadow-lg shadow-black/5 backdrop-blur">
                    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                        <div class="space-y-2">
                            <p class="text-xs uppercase tracking-[0.2em] text-muted-foreground">shadcn-style</p>
                            <h1 class="text-3xl font-bold tracking-tight"><span class="bg-gradient-to-r from-primary to-violet-600 bg-clip-text text-transparent">简易留言板</span><span class="ml-2 text-base font-normal text-muted-foreground/60">v{} ({})</span></h1>
                            <p class="text-sm text-muted-foreground">支持 Markdown 留言，按 Ctrl + Enter 快速提交。最多保留 {} 条。</p>
                        </div>
                        <div class="flex items-center gap-3 self-end sm:self-auto">
                            <span class="inline-flex items-center whitespace-nowrap rounded-full bg-secondary px-3 py-1 text-xs font-medium text-secondary-foreground">共 {} 条留言</span>
                            <span class="inline-flex items-center whitespace-nowrap rounded-full bg-primary/10 px-3 py-1 text-xs font-medium text-primary">历史 {} 条</span>
                            <a href="/dashboard" class="inline-flex h-9 items-center gap-2 whitespace-nowrap rounded-md border border-input bg-background px-3 text-xs font-medium shadow-sm transition hover:bg-accent hover:text-accent-foreground">
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" x2="12" y1="20" y2="10"/><line x1="18" x2="18" y1="20" y2="4"/><line x1="6" x2="6" y1="20" y2="16"/></svg>
                                <span>数据看板</span>
                            </a>
                            <button type="button" id="theme-toggle" class="inline-flex h-9 items-center gap-2 whitespace-nowrap rounded-md border border-input bg-background px-3 text-xs font-medium shadow-sm transition hover:bg-accent hover:text-accent-foreground">
                                <span aria-hidden="true">☀️</span>
                                <span class="theme-toggle-label">亮色</span>
                            </button>
                        </div>
                    </div>
                    <form action="/submit" method="post" class="space-y-3">
                        <div id="markdown-toolbar" class="flex flex-wrap items-center gap-1 rounded-t-lg border border-b-0 border-border bg-muted/40 px-2 py-2">
                            {}
                        </div>
                        <textarea id="message" name="message" rows="5" required placeholder="试试使用 **Markdown** 语法，支持代码块、列表等格式。" class="textarea-glow block w-full rounded-b-lg border border-t-0 border-input bg-background px-4 py-3 text-sm leading-6 text-foreground shadow-sm"></textarea>
                        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-3">
                            <div class="flex flex-1 items-center gap-2 rounded-md border border-input bg-background px-3 py-2.5 text-sm text-foreground shadow-sm focus-within:border-ring focus-within:ring-2 focus-within:ring-ring/40 transition-all hover:border-ring/50">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="opacity-50"><path d="M12 2H2v10l9.29 9.29c.94.94 2.48.94 3.42 0l6.58-6.58c.94-.94.94-2.48 0-3.42L12 2Z"/><path d="M7 7h.01"/></svg>
                                <input type="text" name="tags" placeholder="添加标签（用逗号或空格分隔）" class="flex-1 bg-transparent text-sm placeholder:text-muted-foreground/80 focus:outline-none">
                            </div>
                            <button type="submit" class="btn-glow inline-flex h-10 items-center justify-center whitespace-nowrap rounded-md bg-primary px-6 text-sm font-semibold text-primary-foreground shadow transition-all hover:bg-primary/90 hover:scale-[1.02] active:scale-[0.98]">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="mr-2"><path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/></svg>
                                <span>提交留言</span>
                            </button>
                            <span class="hidden sm:inline-flex items-center gap-1 text-xs text-muted-foreground">
                                <kbd class="px-1.5 py-0.5 rounded border border-border bg-muted font-mono text-[10px]">Ctrl</kbd>
                                <span>+</span>
                                <kbd class="px-1.5 py-0.5 rounded border border-border bg-muted font-mono text-[10px]">Enter</kbd>
                            </span>
                        </div>
                    </form>
                </section>

                <section class="rounded-2xl border border-border bg-card/90 p-5 shadow-sm shadow-black/5 backdrop-blur-sm">
                    <div class="mb-4 flex flex-wrap items-center justify-between gap-2">
                        <div class="flex flex-wrap items-center gap-2">
                            <h2 class="flex items-center gap-2 text-sm font-semibold">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-primary"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                                <span>搜索留言</span>
                            </h2>
                            <span class="hidden text-xs text-muted-foreground sm:inline">支持模糊匹配并保留分页</span>
                        </div>
                        {}
                    </div>
                    <form action="/" method="get" class="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-3">
                        <input type="hidden" name="tag" value="{}">
                        <div class="flex flex-1 items-center gap-2 rounded-md border border-input bg-background px-4 py-2 text-sm text-foreground shadow-sm transition-all focus-within:border-ring focus-within:ring-2 focus-within:ring-ring/40 hover:border-ring/50">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="opacity-50"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                            <input type="search" name="q" value="{}" placeholder="输入关键字" class="flex-1 bg-transparent text-sm placeholder:text-muted-foreground/80 focus:outline-none">
                        </div>
                        <div class="flex items-center gap-2">
                            {}
                            <button type="submit" class="inline-flex h-10 items-center justify-center rounded-md bg-secondary px-4 text-sm font-medium text-secondary-foreground shadow-sm transition hover:bg-secondary/80">搜索</button>
                            {}
                        </div>
                    </form>
                </section>

                <section class="space-y-6">
                    <ul class="space-y-4">
                        {}
                    </ul>
                    {}
                </section>
            </div>
        </main>
    </div>
    <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/dompurify@3.0.6/dist/purify.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js" referrerpolicy="no-referrer"></script>
    <script src="/static/app.js"></script>
</body>
</html>"#,
        escape_attribute(&search_term),
        page_size,
        escape_attribute(&tag_filter),
        tag_sidebar_html,
        VERSION,
        VERSION_DATE,
        MAX_MESSAGES,
        total_messages,
        total_messages_ever,
        toolbar,
        search_display,
        escape_attribute(&tag_filter),
        escape_attribute(&search_term),
        page_size_selector,
        clear_search,
        messages_html,
        pagination_html,
    )
}

fn toolbar_button(action: &str, icon: &str) -> String {
    format!(
        r#"<button type="button" class="toolbar-btn inline-flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-foreground" data-action="{}">{}</button>"#,
        action, icon
    )
}

fn toolbar_divider() -> String {
    r#"<div class="mx-1 h-4 w-[1px] bg-border"></div>"#.to_string()
}

fn render_tag_sidebar(tags: &[TagSidebarItem], tag_filter: &str, page_size: i64) -> String {
    if tags.is_empty() {
        return String::new();
    }

    let clear_link = if !tag_filter.is_empty() {
        format!(
            r#"<a href="/?page_size={}" class="text-[10px] text-muted-foreground hover:text-primary transition-colors">清除</a>"#,
            page_size
        )
    } else {
        String::new()
    };

    let items: Vec<String> = tags.iter().map(|tag| {
        let is_active = tag.id.to_string() == tag_filter;
        let classes = if is_active {
            "group flex items-center justify-between gap-2 py-2 px-2.5 text-xs transition-all rounded-md mb-1 font-medium shadow-sm ring-1 ring-inset"
        } else {
            "group flex items-center justify-between gap-2 py-2 px-2.5 text-xs transition-all rounded-md mb-1 text-muted-foreground hover:bg-muted/60 hover:text-foreground"
        };
        let style = if is_active {
            format!("background-color: {}15; color: {}; --tw-ring-color: {}40;", tag.color, tag.color, tag.color)
        } else {
            String::new()
        };
        let count_style = if is_active {
            format!("background-color: {}; color: white;", tag.color)
        } else {
            "background-color: var(--muted); color: var(--muted-foreground);".to_string()
        };

        format!(r#"<a href="/?tag={}&page_size={}" class="{}" style="{}" title="{}">
                <span class="flex items-center gap-2 min-w-0 flex-1">
                    <span class="inline-block h-1.5 w-1.5 rounded-full flex-shrink-0 shadow-sm" style="background-color: {};"></span>
                    <span class="truncate relative top-[0.5px]">{}</span>
                </span>
                <span class="flex-shrink-0 rounded-md px-1.5 py-0.5 text-[10px] font-bold transition-colors group-hover:bg-background/80" style="{}">{}</span>
            </a>"#,
            tag.id, page_size, classes, style, escape_attribute(&tag.name), tag.color, escape_html(&tag.name), count_style, tag.message_count
        )
    }).collect();

    format!(
        r#"<aside class="fixed left-0 top-24 z-10 w-40 hidden xl:block pl-6">
            <div class="rounded-xl border border-border bg-card/50 shadow-sm backdrop-blur-sm">
                <div class="border-b border-border/50 px-3 py-2.5">
                    <div class="flex items-center justify-between">
                        <h2 class="flex items-center gap-1.5 text-[11px] font-bold text-muted-foreground uppercase tracking-wider">
                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2H2v10l9.29 9.29c.94.94 2.48.94 3.42 0l6.58-6.58c.94-.94.94-2.48 0-3.42L12 2Z"/><path d="M7 7h.01"/></svg>
                            标签
                        </h2>
                        {}
                    </div>
                </div>
                <div class="max-h-[calc(100vh-10rem)] overflow-y-auto p-2 scrollbar-thin scrollbar-thumb-muted scrollbar-track-transparent">
                    {}
                </div>
            </div>
        </aside>"#,
        clear_link,
        items.join("")
    )
}

fn render_messages(
    messages: &[MessageView],
    current_page: i64,
    page_size: i64,
    search_term: &str,
    tag_filter: &str,
) -> String {
    if messages.is_empty() {
        if !search_term.is_empty() {
            return format!(
                r#"<li class="animate-fade-in flex flex-col items-center justify-center gap-6 rounded-2xl border border-dashed border-border bg-gradient-to-b from-card/80 to-card/40 p-16 text-center">
                <div class="relative">
                    <div class="absolute inset-0 animate-pulse rounded-full bg-primary/10 blur-xl"></div>
                    <div class="relative rounded-full bg-gradient-to-br from-muted to-muted/50 p-6 shadow-inner">
                        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-muted-foreground/50">
                            <circle cx="11" cy="11" r="8"/>
                            <path d="m21 21-4.3-4.3"/>
                        </svg>
                    </div>
                </div>
                <div class="space-y-2">
                    <p class="text-base font-semibold text-foreground">没有找到包含 "{}" 的留言</p>
                    <p class="text-sm text-muted-foreground">试试其他关键字，或者清除搜索条件</p>
                </div>
            </li>"#,
                escape_html(search_term)
            );
        } else {
            return r#"<li class="animate-fade-in flex flex-col items-center justify-center gap-6 rounded-2xl border border-dashed border-border bg-gradient-to-b from-card/80 to-card/40 p-16 text-center">
                <div class="relative">
                    <div class="absolute inset-0 animate-pulse rounded-full bg-primary/10 blur-xl"></div>
                    <div class="relative rounded-full bg-gradient-to-br from-muted to-muted/50 p-6 shadow-inner">
                        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="text-muted-foreground/50">
                            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                        </svg>
                    </div>
                </div>
                <div class="space-y-2">
                    <p class="text-base font-semibold text-foreground">还没有留言，快来留下第一条消息吧～</p>
                    <p class="text-sm text-muted-foreground">在上方输入框中写下你的想法</p>
                </div>
            </li>"#.to_string();
        }
    }

    messages
        .iter()
        .map(|msg| render_message_item(msg, current_page, page_size, search_term, tag_filter))
        .collect::<Vec<_>>()
        .join("")
}

fn render_message_item(
    msg: &MessageView,
    current_page: i64,
    page_size: i64,
    search_term: &str,
    tag_filter: &str,
) -> String {
    let tags_html = if !msg.tags.is_empty() {
        let tags: Vec<String> = msg.tags.iter().map(|tag| {
            format!(r#"<a href="/?tag={}&page_size={}" class="tag-item group inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium transition-all hover:brightness-105 hover:shadow-sm active:scale-95" style="background-color: {}15; color: {};">
                    <span class="opacity-40 transition-opacity group-hover:opacity-60">#</span>
                    {}
                </a>"#,
                tag.id, page_size, tag.color, tag.color, escape_html(&tag.name)
            )
        }).collect();
        format!(
            r#"<div class="message-tags flex flex-wrap gap-2 mt-4">{}</div>"#,
            tags.join("")
        )
    } else {
        String::new()
    };

    let search_hidden = if !search_term.is_empty() {
        format!(
            r#"<input type="hidden" name="q" value="{}">"#,
            escape_attribute(search_term)
        )
    } else {
        String::new()
    };

    let tag_hidden = if !tag_filter.is_empty() {
        format!(
            r#"<input type="hidden" name="tag" value="{}">"#,
            escape_attribute(tag_filter)
        )
    } else {
        String::new()
    };
    let page_size_hidden = format!(r#"<input type="hidden" name="page_size" value="{}">"#, page_size);

    let id_str = msg.id.to_string();
    let id_suffix = if id_str.len() > 2 {
        &id_str[id_str.len() - 2..]
    } else {
        &id_str
    };

    let replies_html =
        render_replies_section(&msg.replies, msg.id, current_page, page_size, search_term, tag_filter);

    format!(
        r#"<li class="message-item glass-card-hover group/reply rounded-xl text-card-foreground" data-message-id="{}">
            <div class="flex flex-col gap-4 p-5 sm:flex-row sm:items-start sm:gap-4">
                <div class="hidden sm:block flex-shrink-0 mt-1">
                    <div class="w-10 h-10 text-xs rounded-full bg-gradient-to-br {} flex items-center justify-center text-white font-bold shadow-sm select-none ring-2 ring-background/50">
                        #{}
                    </div>
                </div>
                <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                        <div class="sm:hidden flex-shrink-0">
                            <div class="w-6 h-6 text-[9px] rounded-full bg-gradient-to-br {} flex items-center justify-center text-white font-bold shadow-sm select-none ring-2 ring-background/50">
                                #{}
                            </div>
                        </div>
                        <p class="text-xs font-medium text-muted-foreground/60" data-timestamp="{}">{}</p>
                    </div>
                    <div class="message-content prose prose-slate prose-sm max-w-none dark:prose-invert leading-7 text-foreground/90" data-markdown="{}">{}</div>
                    {}
                </div>
                <form action="/delete" method="post" class="flex shrink-0 items-center justify-end sm:self-start sm:ml-auto">
                    <input type="hidden" name="id" value="{}">
                    <input type="hidden" name="page" value="{}">
                    {}
                    {}
                    {}
                    <button type="submit" class="inline-flex h-8 w-8 items-center justify-center rounded-full text-muted-foreground/60 hover:text-destructive hover:bg-destructive/10 transition-all active:scale-90 opacity-0 group-hover/reply:opacity-100 focus:opacity-100">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                    </button>
                </form>
            </div>
            {}
        </li>"#,
        msg.id,
        msg.avatar_gradient,
        id_suffix,
        msg.avatar_gradient,
        id_suffix,
        msg.created_at,
        msg.display_time,
        escape_attribute(&msg.content),
        escape_html(&msg.content),
        tags_html,
        msg.id,
        current_page,
        page_size_hidden,
        search_hidden,
        tag_hidden,
        replies_html
    )
}

fn render_replies_section(
    replies: &[ReplyView],
    message_id: i64,
    current_page: i64,
    page_size: i64,
    search_term: &str,
    tag_filter: &str,
) -> String {
    let search_hidden = if !search_term.is_empty() {
        format!(
            r#"<input type="hidden" name="q" value="{}">"#,
            escape_attribute(search_term)
        )
    } else {
        String::new()
    };

    let tag_hidden = if !tag_filter.is_empty() {
        format!(
            r#"<input type="hidden" name="tag" value="{}">"#,
            escape_attribute(tag_filter)
        )
    } else {
        String::new()
    };
    let page_size_hidden = format!(r#"<input type="hidden" name="page_size" value="{}">"#, page_size);

    let replies_list = if !replies.is_empty() {
        let items: Vec<String> = replies.iter().map(|reply| {
            let id_str = reply.id.to_string();
            let id_suffix = if id_str.len() > 2 { &id_str[id_str.len()-2..] } else { &id_str };
            format!(r#"<div class="reply-item group/item flex gap-3 py-3 first:pt-0 last:pb-0" data-reply-id="{}">
                    <div class="flex-shrink-0 mt-1">
                        <div class="w-6 h-6 text-[9px] rounded-full bg-gradient-to-br {} flex items-center justify-center text-white font-bold shadow-sm select-none ring-2 ring-background/50">
                            #{}
                        </div>
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-[10px] font-medium text-muted-foreground mb-1" data-timestamp="{}">{}</p>
                        <div class="reply-content prose prose-slate max-w-none text-xs dark:prose-invert" data-markdown="{}">{}</div>
                    </div>
                    <form action="/delete-reply" method="post" class="flex-shrink-0 self-start opacity-0 group-hover/item:opacity-100 transition-opacity">
                        <input type="hidden" name="id" value="{}">
                        <input type="hidden" name="page" value="{}">
                        {}
                        {}
                        {}
                        <button type="submit" class="inline-flex h-6 w-6 items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition">
                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                        </button>
                    </form>
                </div>"#,
                reply.id,
                reply.avatar_gradient,
                id_suffix,
                reply.created_at,
                reply.display_time,
                escape_attribute(&reply.content),
                escape_html(&reply.content),
                reply.id,
                current_page,
                page_size_hidden,
                search_hidden,
                tag_hidden
            )
        }).collect();
        format!(
            r#"<div class="replies-list divide-y divide-border/50">{}</div>"#,
            items.join("")
        )
    } else {
        String::new()
    };

    let mt_class = if !replies.is_empty() { "mt-3" } else { "" };
    let reply_count = if !replies.is_empty() {
        format!(
            r#"<span class="text-[10px] text-muted-foreground/70">({})</span>"#,
            replies.len()
        )
    } else {
        String::new()
    };

    format!(
        r#"<div class="replies-section border-t border-border/50 mx-5 px-0 pb-5 pt-4">
            {}
            <div class="reply-form-container {}">
                <button type="button" class="reply-toggle-btn inline-flex items-center gap-1.5 text-xs text-muted-foreground hover:text-primary transition" data-message-id="{}">
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m3 21 1.9-5.7a8.5 8.5 0 1 1 3.8 3.8z"/></svg>
                    <span>添加答复</span>
                    {}
                </button>
                <form action="/reply" method="post" class="reply-form hidden mt-3" data-message-id="{}">
                    <input type="hidden" name="message_id" value="{}">
                    <input type="hidden" name="page" value="{}">
                    {}
                    {}
                    {}
                    <div class="flex gap-2">
                        <textarea name="content" rows="2" required placeholder="输入答复内容..." class="flex-1 rounded-lg border border-input bg-background px-3 py-2 text-xs leading-5 text-foreground shadow-sm focus:border-ring focus:outline-none focus:ring-2 focus:ring-ring/40 resize-none"></textarea>
                        <div class="flex flex-col gap-1">
                            <button type="submit" class="inline-flex h-8 items-center justify-center rounded-md bg-primary px-3 text-xs font-medium text-primary-foreground shadow transition hover:bg-primary/90">
                                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/></svg>
                            </button>
                            <button type="button" class="reply-cancel-btn inline-flex h-8 items-center justify-center rounded-md border border-input bg-background px-3 text-xs font-medium text-muted-foreground shadow-sm transition hover:bg-accent hover:text-accent-foreground">
                                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                            </button>
                        </div>
                    </div>
                </form>
            </div>
        </div>"#,
        replies_list,
        mt_class,
        message_id,
        reply_count,
        message_id,
        message_id,
        current_page,
        page_size_hidden,
        search_hidden,
        tag_hidden
    )
}

fn render_pagination(
    current_page: i64,
    page_size: i64,
    total_pages: i64,
    search_term: &str,
    tag_filter: &str,
    pages: &[String],
    prev_page: i64,
    next_page: i64,
) -> String {
    let search_param = if !search_term.is_empty() {
        format!("&q={}", urlencoding::encode(search_term))
    } else {
        String::new()
    };

    let tag_param = if !tag_filter.is_empty() {
        format!("&tag={}", tag_filter)
    } else {
        String::new()
    };
    let page_size_param = format!("&page_size={}", page_size);

    let page_numbers: Vec<String> = pages.iter().map(|p| {
        if p == "..." {
            r#"<span class="px-2 text-muted-foreground">...</span>"#.to_string()
        } else {
            let page_num: i64 = p.parse().unwrap_or(1);
            let is_active = page_num == current_page;
            let active_class = if is_active { "bg-primary text-primary-foreground shadow hover:bg-primary/90" } else { "" };
            format!(r#"<a href="/?page={}{}{}{}" class="inline-flex h-9 min-w-[2.25rem] items-center justify-center rounded-md border border-input px-3 text-xs font-medium transition hover:bg-accent hover:text-accent-foreground {}">{}</a>"#,
                page_num, page_size_param, search_param, tag_param, active_class, p
            )
        }
    }).collect();

    let prev_disabled = if current_page == 1 {
        "cursor-not-allowed bg-muted text-muted-foreground"
    } else {
        ""
    };
    let next_disabled = if current_page == total_pages {
        "cursor-not-allowed bg-muted text-muted-foreground"
    } else {
        ""
    };

    format!(
        r#"<nav class="flex flex-col gap-3 rounded-xl border border-border bg-card/70 p-4 shadow-sm sm:flex-row sm:items-center sm:justify-between">
            <div class="text-xs text-muted-foreground">第 {} / {} 页</div>
            <div class="flex flex-wrap items-center gap-2">
                <a href="/?page={}{}{}{}" class="inline-flex h-9 min-w-[2.25rem] items-center justify-center rounded-md border border-input px-3 text-xs font-medium transition hover:bg-accent hover:text-accent-foreground {}">上一页</a>
                <div class="flex items-center gap-1">{}</div>
                <a href="/?page={}{}{}{}" class="inline-flex h-9 min-w-[2.25rem] items-center justify-center rounded-md border border-input px-3 text-xs font-medium transition hover:bg-accent hover:text-accent-foreground {}">下一页</a>
            </div>
        </nav>"#,
        current_page,
        total_pages,
        prev_page,
        page_size_param,
        search_param,
        tag_param,
        prev_disabled,
        page_numbers.join(""),
        next_page,
        page_size_param,
        search_param,
        tag_param,
        next_disabled
    )
}

fn render_page_size_selector(current_page_size: i64) -> String {
    let options = PAGE_SIZE_OPTIONS
        .iter()
        .map(|size| {
            let selected = if *size == current_page_size {
                " selected"
            } else {
                ""
            };
            format!(r#"<option value="{}"{}>{} / 页</option>"#, size, selected, size)
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"<label class="inline-flex items-center gap-2 rounded-md border border-input bg-background px-3 py-2 text-xs text-muted-foreground shadow-sm">
            <span>显示</span>
            <select name="page_size" class="bg-transparent text-foreground focus:outline-none">
                {}
            </select>
        </label>"#,
        options
    )
}

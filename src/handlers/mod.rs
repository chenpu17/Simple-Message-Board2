mod api;
mod dashboard;
mod home;

use crate::config::{MAX_MESSAGES, MAX_MESSAGE_LENGTH, MAX_REPLY_LENGTH, MAX_TAG_NAME_LENGTH};
use crate::db::Repository;
use crate::utils::*;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use tracing::{error, warn};

pub use api::{api_messages, api_tags};
pub use dashboard::dashboard;
pub use home::home;

#[derive(Deserialize)]
pub struct SubmitForm {
    message: String,
    tags: String,
}

#[derive(Deserialize)]
pub struct DeleteForm {
    id: i64,
    page: Option<i64>,
    q: Option<String>,
    tag: Option<String>,
}

#[derive(Deserialize)]
pub struct ReplyForm {
    message_id: i64,
    content: String,
    page: Option<i64>,
    q: Option<String>,
    tag: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteReplyForm {
    id: i64,
    page: Option<i64>,
    q: Option<String>,
    tag: Option<String>,
}

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

pub async fn submit_message(
    repo: web::Data<Repository>,
    form: web::Form<SubmitForm>,
) -> HttpResponse {
    let content = form.message.trim();

    // 输入验证：检查留言是否为空
    if content.is_empty() {
        return HttpResponse::Found()
            .insert_header(("Location", "/"))
            .finish();
    }

    // 输入验证：检查留言长度（按Unicode字符计数）
    if content.chars().count() > MAX_MESSAGE_LENGTH {
        return HttpResponse::Found()
            .insert_header(("Location", "/"))
            .finish();
    }

    let created_at = now_iso();

    // 创建留言
    let message_id = match repo.create_message(content, &created_at).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to create message: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // 更新统计
    if let Err(e) = repo.increment_stat("total_messages_ever").await {
        warn!("Failed to increment total_messages_ever stat: {}", e);
    }
    if let Err(e) = repo.update_daily_stats(&today_date(), true).await {
        warn!("Failed to update daily stats: {}", e);
    }

    // 处理标签
    if !form.tags.is_empty() {
        let tag_names: Vec<&str> = form
            .tags
            .split([',', ' ', '，'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for name in tag_names {
            // 输入验证：标签名长度限制（按Unicode字符计数）
            if name.chars().count() <= MAX_TAG_NAME_LENGTH {
                if let Ok(tag) = repo.get_or_create_tag(name).await {
                    if let Err(e) = repo.add_tag_to_message(message_id, tag.id).await {
                        warn!("Failed to add tag to message: {}", e);
                    }
                }
            }
        }
    }

    // 清理旧留言
    if let Err(e) = repo.cleanup_old_messages(MAX_MESSAGES).await {
        warn!("Failed to cleanup old messages: {}", e);
    }

    HttpResponse::Found()
        .insert_header(("Location", "/"))
        .finish()
}

pub async fn delete_message(
    repo: web::Data<Repository>,
    form: web::Form<DeleteForm>,
) -> HttpResponse {
    if let Err(e) = repo.delete_message(form.id).await {
        warn!("Failed to delete message {}: {}", form.id, e);
    }

    let redirect = build_redirect_path(form.page, form.q.as_deref(), form.tag.as_deref());
    HttpResponse::Found()
        .insert_header(("Location", redirect))
        .finish()
}

pub async fn submit_reply(repo: web::Data<Repository>, form: web::Form<ReplyForm>) -> HttpResponse {
    let content = form.content.trim();

    // 输入验证：检查内容是否为空
    if content.is_empty() {
        let redirect = build_redirect_path(form.page, form.q.as_deref(), form.tag.as_deref());
        return HttpResponse::Found()
            .insert_header(("Location", redirect))
            .finish();
    }

    // 输入验证：检查内容长度（按Unicode字符计数）
    if content.chars().count() > MAX_REPLY_LENGTH {
        let redirect = build_redirect_path(form.page, form.q.as_deref(), form.tag.as_deref());
        return HttpResponse::Found()
            .insert_header(("Location", redirect))
            .finish();
    }

    let created_at = now_iso();

    if repo
        .create_reply(form.message_id, content, &created_at)
        .await
        .is_ok()
    {
        if let Err(e) = repo.update_daily_stats(&today_date(), false).await {
            warn!("Failed to update daily stats for reply: {}", e);
        }
    }

    let redirect = build_redirect_path(form.page, form.q.as_deref(), form.tag.as_deref());
    HttpResponse::Found()
        .insert_header(("Location", redirect))
        .finish()
}

pub async fn delete_reply(
    repo: web::Data<Repository>,
    form: web::Form<DeleteReplyForm>,
) -> HttpResponse {
    if let Err(e) = repo.delete_reply(form.id).await {
        warn!("Failed to delete reply {}: {}", form.id, e);
    }

    let redirect = build_redirect_path(form.page, form.q.as_deref(), form.tag.as_deref());
    HttpResponse::Found()
        .insert_header(("Location", redirect))
        .finish()
}

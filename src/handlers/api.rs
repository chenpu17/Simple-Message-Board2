use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use crate::db::Repository;

#[derive(Deserialize)]
pub struct MessagesQuery {
    since_id: Option<i64>,
    limit: Option<i64>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    id: i64,
    content: String,
    created_at: String,
    tags: Vec<TagResponse>,
    replies: Vec<ReplyResponse>,
}

#[derive(Serialize)]
pub struct TagResponse {
    id: i64,
    name: String,
    color: String,
}

#[derive(Serialize)]
pub struct ReplyResponse {
    id: i64,
    content: String,
    created_at: String,
}

#[derive(Serialize)]
pub struct TagWithCountResponse {
    id: i64,
    name: String,
    color: String,
    count: i64,
}

pub async fn api_messages(
    repo: web::Data<Repository>,
    query: web::Query<MessagesQuery>,
) -> HttpResponse {
    let since_id = query.since_id.unwrap_or(0);
    let limit = query.limit.unwrap_or(20).min(100);

    match repo.get_messages_since(since_id, limit).await {
        Ok(messages) => {
            let response: Vec<MessageResponse> = messages.iter().map(|m| {
                MessageResponse {
                    id: m.id,
                    content: m.content.clone(),
                    created_at: m.created_at.clone(),
                    tags: m.tags.iter().map(|t| TagResponse {
                        id: t.id,
                        name: t.name.clone(),
                        color: t.color.clone(),
                    }).collect(),
                    replies: m.replies.iter().map(|r| ReplyResponse {
                        id: r.id,
                        content: r.content.clone(),
                        created_at: r.created_at.clone(),
                    }).collect(),
                }
            }).collect();

            HttpResponse::Ok()
                .content_type("application/json")
                .json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn api_tags(repo: web::Data<Repository>) -> HttpResponse {
    match repo.get_tags_with_count().await {
        Ok(tags) => {
            let response: Vec<TagWithCountResponse> = tags.iter().map(|t| {
                TagWithCountResponse {
                    id: t.id,
                    name: t.name.clone(),
                    color: t.color.clone(),
                    count: t.count,
                }
            }).collect();

            HttpResponse::Ok()
                .content_type("application/json")
                .json(response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

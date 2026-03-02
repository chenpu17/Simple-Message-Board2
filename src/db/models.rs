use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub content: String,
    pub created_at: String,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Message {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Message {
            id: row.try_get("id")?,
            content: row.try_get("content")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Tag {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Tag {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            color: row.try_get("color")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTag {
    pub message_id: i64,
    pub tag_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply {
    pub id: i64,
    pub message_id: i64,
    pub content: String,
    pub created_at: String,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for Reply {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Reply {
            id: row.try_get("id")?,
            message_id: row.try_get("message_id")?,
            content: row.try_get("content")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub key: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: String,
    pub message_count: i64,
    pub reply_count: i64,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for DailyStat {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(DailyStat {
            date: row.try_get("date")?,
            message_count: row.try_get("message_count")?,
            reply_count: row.try_get("reply_count")?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageWithTags {
    pub id: i64,
    pub content: String,
    pub created_at: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageWithDetails {
    pub id: i64,
    pub content: String,
    pub created_at: String,
    pub tags: Vec<Tag>,
    pub replies: Vec<Reply>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TagWithCount {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub count: i64,
}

impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for TagWithCount {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(TagWithCount {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            color: row.try_get("color")?,
            count: row.try_get("count")?,
        })
    }
}

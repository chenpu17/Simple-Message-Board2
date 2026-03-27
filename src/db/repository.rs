use crate::db::models::*;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::str::FromStr;

/// 转义LIKE查询中的通配符，防止通配符注入
fn escape_like_pattern(query: &str) -> String {
    query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[derive(Clone)]
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        // 配置SQLite连接选项：启用外键约束（对所有连接生效）
        let options = SqliteConnectOptions::from_str(database_url)?.pragma("foreign_keys", "ON");

        // 创建连接池，确保每个连接都应用PRAGMA
        let pool = SqlitePoolOptions::new().connect_with(options).await?;

        let repo = Repository { pool };
        repo.init_tables().await?;
        repo.create_indexes().await?;
        Ok(repo)
    }

    async fn create_indexes(&self) -> Result<(), sqlx::Error> {
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_messages_source_ip ON messages(source_ip)",
            "CREATE INDEX IF NOT EXISTS idx_replies_message_id ON replies(message_id)",
            "CREATE INDEX IF NOT EXISTS idx_replies_created_at ON replies(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_message_tags_tag_id ON message_tags(tag_id)",
            "CREATE INDEX IF NOT EXISTS idx_daily_stats_date ON daily_stats(date)",
            "CREATE INDEX IF NOT EXISTS idx_daily_ip_stats_date ON daily_ip_stats(date)",
            "CREATE INDEX IF NOT EXISTS idx_daily_ip_stats_source_ip ON daily_ip_stats(source_ip)",
        ];

        for idx in indexes {
            sqlx::query(idx).execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn init_tables(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                source_ip TEXT NOT NULL DEFAULT ''
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        let message_columns = sqlx::query("PRAGMA table_info(messages)")
            .fetch_all(&self.pool)
            .await?;
        let has_source_ip = message_columns.iter().any(|column| {
            column
                .try_get::<String, _>("name")
                .map(|name| name == "source_ip")
                .unwrap_or(false)
        });
        if !has_source_ip {
            sqlx::query("ALTER TABLE messages ADD COLUMN source_ip TEXT NOT NULL DEFAULT ''")
                .execute(&self.pool)
                .await?;
        }

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                color TEXT NOT NULL DEFAULT '#3b82f6'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS message_tags (
                message_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (message_id, tag_id),
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS replies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS stats (
                key TEXT PRIMARY KEY,
                value INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS daily_stats (
                date TEXT PRIMARY KEY,
                message_count INTEGER NOT NULL DEFAULT 0,
                reply_count INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS daily_ip_stats (
                date TEXT NOT NULL,
                source_ip TEXT NOT NULL,
                message_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (date, source_ip)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Message operations
    pub async fn create_message_with_ip(
        &self,
        content: &str,
        created_at: &str,
        source_ip: &str,
    ) -> Result<i64, sqlx::Error> {
        let result =
            sqlx::query("INSERT INTO messages (content, created_at, source_ip) VALUES (?, ?, ?)")
                .bind(content)
                .bind(created_at)
                .bind(source_ip)
                .execute(&self.pool)
                .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn create_message(
        &self,
        content: &str,
        created_at: &str,
    ) -> Result<i64, sqlx::Error> {
        self.create_message_with_ip(content, created_at, "").await
    }

    pub async fn get_messages(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        sqlx::query_as::<_, Message>(
            "SELECT id, content, created_at FROM messages ORDER BY id DESC LIMIT ? OFFSET ?",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete_message(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count_messages(&self) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
            .fetch_one(&self.pool)
            .await?;
        Ok(result.0)
    }

    pub async fn count_search_messages(&self, query: &str) -> Result<i64, sqlx::Error> {
        let search_pattern = format!("%{}%", escape_like_pattern(query));
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM messages WHERE content LIKE ? ESCAPE '\\'")
                .bind(&search_pattern)
                .fetch_one(&self.pool)
                .await?;
        Ok(result.0)
    }

    pub async fn count_messages_by_tag(&self, tag_id: i64) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM message_tags WHERE tag_id = ?")
            .bind(tag_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(result.0)
    }

    // Tag operations
    pub async fn get_or_create_tag(&self, name: &str) -> Result<Tag, sqlx::Error> {
        if let Some(tag) =
            sqlx::query_as::<_, Tag>("SELECT id, name, color FROM tags WHERE name = ?")
                .bind(name)
                .fetch_optional(&self.pool)
                .await?
        {
            return Ok(tag);
        }

        let color = Self::generate_tag_color(name);
        sqlx::query("INSERT INTO tags (name, color) VALUES (?, ?)")
            .bind(name)
            .bind(&color)
            .execute(&self.pool)
            .await?;

        sqlx::query_as::<_, Tag>("SELECT id, name, color FROM tags WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await
    }

    fn generate_tag_color(name: &str) -> String {
        let colors = [
            "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6", "#ec4899", "#06b6d4", "#84cc16",
        ];
        let hash = name
            .bytes()
            .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
        colors[hash % colors.len()].to_string()
    }

    pub async fn add_tag_to_message(
        &self,
        message_id: i64,
        tag_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR IGNORE INTO message_tags (message_id, tag_id) VALUES (?, ?)")
            .bind(message_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_tags_with_count(&self) -> Result<Vec<TagWithCount>, sqlx::Error> {
        sqlx::query_as::<_, TagWithCount>(
            r#"
            SELECT t.id, t.name, t.color, COUNT(mt.message_id) as count
            FROM tags t
            LEFT JOIN message_tags mt ON t.id = mt.tag_id
            GROUP BY t.id
            ORDER BY t.name
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    // Reply operations
    pub async fn create_reply(
        &self,
        message_id: i64,
        content: &str,
        created_at: &str,
    ) -> Result<i64, sqlx::Error> {
        let result =
            sqlx::query("INSERT INTO replies (message_id, content, created_at) VALUES (?, ?, ?)")
                .bind(message_id)
                .bind(content)
                .bind(created_at)
                .execute(&self.pool)
                .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn delete_reply(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM replies WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Stats operations
    pub async fn get_stat(&self, key: &str) -> Result<i64, sqlx::Error> {
        let result: Option<(i64,)> = sqlx::query_as("SELECT value FROM stats WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result.map(|r| r.0).unwrap_or(0))
    }

    pub async fn increment_stat(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO stats (key, value) VALUES (?, 1) ON CONFLICT(key) DO UPDATE SET value = value + 1"
        )
        .bind(key)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_total_replies(&self) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM replies")
            .fetch_one(&self.pool)
            .await?;
        Ok(result.0)
    }

    pub async fn get_daily_stats(&self) -> Result<Vec<DailyStat>, sqlx::Error> {
        sqlx::query_as::<_, DailyStat>(
            "SELECT date, message_count, reply_count FROM daily_stats ORDER BY date DESC LIMIT 30",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_daily_stats(
        &self,
        date: &str,
        is_message: bool,
    ) -> Result<(), sqlx::Error> {
        if is_message {
            sqlx::query(
                r#"
                INSERT INTO daily_stats (date, message_count, reply_count) VALUES (?, 1, 0)
                ON CONFLICT(date) DO UPDATE SET message_count = message_count + 1
                "#,
            )
            .bind(date)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO daily_stats (date, message_count, reply_count) VALUES (?, 0, 1)
                ON CONFLICT(date) DO UPDATE SET reply_count = reply_count + 1
                "#,
            )
            .bind(date)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn update_daily_ip_stats(
        &self,
        date: &str,
        source_ip: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO daily_ip_stats (date, source_ip, message_count) VALUES (?, ?, 1)
            ON CONFLICT(date, source_ip) DO UPDATE SET message_count = message_count + 1
            "#,
        )
        .bind(date)
        .bind(source_ip)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_daily_ip_stats(&self, limit: i64) -> Result<Vec<DailyIpStat>, sqlx::Error> {
        sqlx::query_as::<_, DailyIpStat>(
            r#"
            SELECT date, source_ip, message_count
            FROM daily_ip_stats
            ORDER BY date DESC, message_count DESC, source_ip ASC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_unique_source_ip_count(&self) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT source_ip) FROM messages WHERE source_ip IS NOT NULL AND source_ip != ''",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(result.0)
    }

    pub async fn get_database_size_bytes(&self) -> Result<i64, sqlx::Error> {
        let page_count: (i64,) = sqlx::query_as("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await?;
        let page_size: (i64,) = sqlx::query_as("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await?;
        Ok(page_count.0.saturating_mul(page_size.0))
    }

    // Cleanup old messages
    pub async fn cleanup_old_messages(&self, max_count: i64) -> Result<(), sqlx::Error> {
        let count = self.count_messages().await?;
        if count > max_count {
            let delete_count = count - max_count;
            sqlx::query(
                "DELETE FROM messages WHERE id IN (SELECT id FROM messages ORDER BY id ASC LIMIT ?)"
            )
            .bind(delete_count)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    // API: Get messages with details
    pub async fn get_messages_since(
        &self,
        since_id: i64,
        limit: i64,
    ) -> Result<Vec<MessageWithDetails>, sqlx::Error> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT id, content, created_at FROM messages WHERE id > ? ORDER BY id ASC LIMIT ?",
        )
        .bind(since_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        // Batch fetch tags and replies to avoid N+1
        let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
        let all_tags = self.get_tags_for_messages_batch(&message_ids).await?;
        let all_replies = self.get_replies_for_messages_batch(&message_ids).await?;

        let result: Vec<MessageWithDetails> = messages
            .into_iter()
            .map(|msg| {
                let tags = all_tags.get(&msg.id).cloned().unwrap_or_default();
                let replies = all_replies.get(&msg.id).cloned().unwrap_or_default();
                MessageWithDetails {
                    id: msg.id,
                    content: msg.content,
                    created_at: msg.created_at,
                    tags,
                    replies,
                }
            })
            .collect();

        Ok(result)
    }

    // Batch operations to fix N+1 queries
    pub async fn get_tags_for_messages_batch(
        &self,
        message_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, Vec<Tag>>, sqlx::Error> {
        if message_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let placeholders: Vec<String> = message_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT t.id, t.name, t.color, mt.message_id FROM tags t JOIN message_tags mt ON t.id = mt.tag_id WHERE mt.message_id IN ({})",
            placeholders.join(",")
        );

        let mut query = sqlx::query(&sql);
        for &id in message_ids {
            query = query.bind(id);
        }

        let rows = query.fetch_all(&self.pool).await?;
        let mut result: std::collections::HashMap<i64, Vec<Tag>> = std::collections::HashMap::new();

        for row in rows {
            let message_id: i64 = row.try_get("message_id")?;
            let tag = Tag {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                color: row.try_get("color")?,
            };
            result.entry(message_id).or_default().push(tag);
        }

        Ok(result)
    }

    pub async fn get_replies_for_messages_batch(
        &self,
        message_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, Vec<Reply>>, sqlx::Error> {
        if message_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let placeholders: Vec<String> = message_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT id, message_id, content, created_at FROM replies WHERE message_id IN ({}) ORDER BY id ASC",
            placeholders.join(",")
        );

        let mut query = sqlx::query(&sql);
        for &id in message_ids {
            query = query.bind(id);
        }

        let rows = query.fetch_all(&self.pool).await?;
        let mut result: std::collections::HashMap<i64, Vec<Reply>> =
            std::collections::HashMap::new();

        for row in rows {
            let message_id: i64 = row.try_get("message_id")?;
            let reply = Reply {
                id: row.try_get("id")?,
                message_id,
                content: row.try_get("content")?,
                created_at: row.try_get("created_at")?,
            };
            result.entry(message_id).or_default().push(reply);
        }

        Ok(result)
    }

    // Dashboard real data queries
    pub async fn get_average_message_length(&self) -> Result<f64, sqlx::Error> {
        let result: Option<(f64,)> = sqlx::query_as("SELECT AVG(LENGTH(content)) FROM messages")
            .fetch_optional(&self.pool)
            .await?;
        Ok(result.map(|r| r.0).unwrap_or(0.0))
    }

    pub async fn get_hourly_distribution(&self) -> Result<Vec<i64>, sqlx::Error> {
        let rows: Vec<(i32, i64)> = sqlx::query_as(
            r#"
            SELECT CAST(strftime('%H', created_at) AS INTEGER) as hour, COUNT(*) as count
            FROM messages
            GROUP BY hour
            ORDER BY hour
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut hourly = vec![0i64; 24];
        for (hour, count) in rows {
            if (0..24).contains(&hour) {
                hourly[hour as usize] = count;
            }
        }
        Ok(hourly)
    }

    pub async fn get_top_messages_by_replies(
        &self,
        limit: i64,
    ) -> Result<Vec<(String, i64)>, sqlx::Error> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT m.content, COUNT(r.id) as reply_count
            FROM messages m
            JOIN replies r ON m.id = r.message_id
            GROUP BY m.id
            ORDER BY reply_count DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_messages_with_tags_batch(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<MessageWithTags>, sqlx::Error> {
        let messages = self.get_messages(page, per_page).await?;
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
        let all_tags = self.get_tags_for_messages_batch(&message_ids).await?;

        let result: Vec<MessageWithTags> = messages
            .into_iter()
            .map(|msg| {
                let tags = all_tags.get(&msg.id).cloned().unwrap_or_default();
                MessageWithTags {
                    id: msg.id,
                    content: msg.content,
                    created_at: msg.created_at,
                    tags,
                }
            })
            .collect();

        Ok(result)
    }

    pub async fn search_messages_with_tags_batch(
        &self,
        query: &str,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<MessageWithTags>, sqlx::Error> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", escape_like_pattern(query));

        let messages = sqlx::query_as::<_, Message>(
            "SELECT id, content, created_at FROM messages WHERE content LIKE ? ESCAPE '\\' ORDER BY id DESC LIMIT ? OFFSET ?"
        )
        .bind(&search_pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
        let all_tags = self.get_tags_for_messages_batch(&message_ids).await?;

        let result: Vec<MessageWithTags> = messages
            .into_iter()
            .map(|msg| {
                let tags = all_tags.get(&msg.id).cloned().unwrap_or_default();
                MessageWithTags {
                    id: msg.id,
                    content: msg.content,
                    created_at: msg.created_at,
                    tags,
                }
            })
            .collect();

        Ok(result)
    }

    pub async fn get_messages_by_tag_with_tags_batch(
        &self,
        tag_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<MessageWithTags>, sqlx::Error> {
        let offset = (page - 1) * per_page;

        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT m.id, m.content, m.created_at
            FROM messages m
            JOIN message_tags mt ON m.id = mt.message_id
            WHERE mt.tag_id = ?
            ORDER BY m.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(tag_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let message_ids: Vec<i64> = messages.iter().map(|m| m.id).collect();
        let all_tags = self.get_tags_for_messages_batch(&message_ids).await?;

        let result: Vec<MessageWithTags> = messages
            .into_iter()
            .map(|msg| {
                let tags = all_tags.get(&msg.id).cloned().unwrap_or_default();
                MessageWithTags {
                    id: msg.id,
                    content: msg.content,
                    created_at: msg.created_at,
                    tags,
                }
            })
            .collect();

        Ok(result)
    }
}

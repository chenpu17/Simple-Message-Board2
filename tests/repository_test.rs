//! Repository 数据库操作测试
//!
//! 测试所有数据库 CRUD 操作

use message_board::db::Repository;

/// 创建测试用的内存数据库
async fn create_test_repo() -> Repository {
    Repository::new("sqlite::memory:")
        .await
        .expect("Failed to create test database")
}

/// 测试数据库初始化
#[actix_rt::test]
async fn test_repository_init() {
    let repo = create_test_repo().await;

    // 验证初始状态
    let count = repo.count_messages().await.unwrap();
    assert_eq!(count, 0);
}

/// 测试创建留言
#[actix_rt::test]
async fn test_create_message() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id = repo.create_message("Hello, world!", &created_at)
        .await
        .unwrap();

    assert!(id > 0);

    let count = repo.count_messages().await.unwrap();
    assert_eq!(count, 1);
}

/// 测试创建多条留言
#[actix_rt::test]
async fn test_create_multiple_messages() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    for i in 1..=10 {
        let id = repo.create_message(&format!("Message {}", i), &created_at)
            .await
            .unwrap();
        assert_eq!(id, i);
    }

    let count = repo.count_messages().await.unwrap();
    assert_eq!(count, 10);
}

/// 测试删除留言
#[actix_rt::test]
async fn test_delete_message() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id = repo.create_message("To be deleted", &created_at)
        .await
        .unwrap();

    // 确认创建成功
    let count = repo.count_messages().await.unwrap();
    assert_eq!(count, 1);

    // 删除留言
    repo.delete_message(id).await.unwrap();

    // 确认删除成功
    let count = repo.count_messages().await.unwrap();
    assert_eq!(count, 0);
}

/// 测试删除不存在的留言
#[actix_rt::test]
async fn test_delete_nonexistent_message() {
    let repo = create_test_repo().await;

    // 删除不存在的留言应该不会报错
    let result = repo.delete_message(999).await;
    assert!(result.is_ok());
}

/// 测试获取留言（分页）
#[actix_rt::test]
async fn test_get_messages_pagination() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建25条留言
    for i in 1..=25 {
        repo.create_message(&format!("Message {}", i), &created_at)
            .await
            .unwrap();
    }

    // 测试第一页
    let page1 = repo.get_messages(1, 10).await.unwrap();
    assert_eq!(page1.len(), 10);

    // 测试第二页
    let page2 = repo.get_messages(2, 10).await.unwrap();
    assert_eq!(page2.len(), 10);

    // 测试第三页
    let page3 = repo.get_messages(3, 10).await.unwrap();
    assert_eq!(page3.len(), 5);

    // 测试空页
    let page4 = repo.get_messages(4, 10).await.unwrap();
    assert_eq!(page4.len(), 0);
}

/// 测试搜索留言
#[actix_rt::test]
async fn test_search_messages() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    repo.create_message("Hello world", &created_at)
        .await
        .unwrap();
    repo.create_message("Rust programming", &created_at)
        .await
        .unwrap();
    repo.create_message("Web development", &created_at)
        .await
        .unwrap();

    // 搜索 "rust"（SQLite LIKE 默认不区分大小写）
    let count = repo.count_search_messages("rust").await.unwrap();
    assert_eq!(count, 1);

    // 搜索 "o" (应该匹配 "Hello world" 和 "Web development")
    // 注意: SQLite LIKE 不区分大小写，所以也会匹配 "Rust programming" 中的 "o"
    let count = repo.count_search_messages("o").await.unwrap();
    assert!(count >= 2); // 至少匹配 2 条

    // 搜索不存在的内容
    let count = repo.count_search_messages("xyz").await.unwrap();
    assert_eq!(count, 0);
}

/// 测试搜索特殊字符
#[actix_rt::test]
async fn test_search_special_characters() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    repo.create_message("100% complete", &created_at)
        .await
        .unwrap();
    repo.create_message("test_value", &created_at)
        .await
        .unwrap();

    // 搜索包含 % 的内容
    let count = repo.count_search_messages("100%").await.unwrap();
    assert_eq!(count, 1);

    // 搜索包含 _ 的内容
    let count = repo.count_search_messages("test_").await.unwrap();
    assert_eq!(count, 1);
}

/// 测试创建标签
#[actix_rt::test]
async fn test_create_tag() {
    let repo = create_test_repo().await;

    let tag = repo.get_or_create_tag("rust").await.unwrap();

    assert!(tag.id > 0);
    assert_eq!(tag.name, "rust");
    assert!(!tag.color.is_empty());
}

/// 测试获取或创建标签（幂等性）
#[actix_rt::test]
async fn test_get_or_create_tag_idempotent() {
    let repo = create_test_repo().await;

    let tag1 = repo.get_or_create_tag("rust").await.unwrap();
    let tag2 = repo.get_or_create_tag("rust").await.unwrap();

    // 相同名称的标签应该返回相同的 ID
    assert_eq!(tag1.id, tag2.id);
    assert_eq!(tag1.name, tag2.name);
}

/// 测试标签颜色生成
#[actix_rt::test]
async fn test_tag_color_generation() {
    let repo = create_test_repo().await;

    let tag1 = repo.get_or_create_tag("rust").await.unwrap();
    let tag2 = repo.get_or_create_tag("web").await.unwrap();

    // 不同标签应该有不同的颜色（虽然可能偶然相同）
    assert!(!tag1.color.is_empty());
    assert!(!tag2.color.is_empty());
}

/// 测试添加标签到留言
#[actix_rt::test]
async fn test_add_tag_to_message() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let msg_id = repo.create_message("Test message", &created_at)
        .await
        .unwrap();

    let tag = repo.get_or_create_tag("test").await.unwrap();
    repo.add_tag_to_message(msg_id, tag.id).await.unwrap();

    let tags = repo.get_tags_with_count().await.unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "test");
    assert_eq!(tags[0].count, 1);
}

/// 测试添加多个标签到留言
#[actix_rt::test]
async fn test_add_multiple_tags_to_message() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let msg_id = repo.create_message("Test message", &created_at)
        .await
        .unwrap();

    let tag1 = repo.get_or_create_tag("rust").await.unwrap();
    let tag2 = repo.get_or_create_tag("web").await.unwrap();
    let tag3 = repo.get_or_create_tag("api").await.unwrap();

    repo.add_tag_to_message(msg_id, tag1.id).await.unwrap();
    repo.add_tag_to_message(msg_id, tag2.id).await.unwrap();
    repo.add_tag_to_message(msg_id, tag3.id).await.unwrap();

    let tags = repo.get_tags_with_count().await.unwrap();
    assert_eq!(tags.len(), 3);
}

/// 测试重复添加标签（幂等性）
#[actix_rt::test]
async fn test_add_tag_to_message_idempotent() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let msg_id = repo.create_message("Test message", &created_at)
        .await
        .unwrap();

    let tag = repo.get_or_create_tag("test").await.unwrap();

    // 添加两次相同的标签
    repo.add_tag_to_message(msg_id, tag.id).await.unwrap();
    repo.add_tag_to_message(msg_id, tag.id).await.unwrap();

    // 标签计数应该仍然是 1
    let tags = repo.get_tags_with_count().await.unwrap();
    assert_eq!(tags[0].count, 1);
}

/// 测试获取标签统计
#[actix_rt::test]
async fn test_get_tags_with_count() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建留言并添加标签
    let msg1 = repo.create_message("Message 1", &created_at).await.unwrap();
    let msg2 = repo.create_message("Message 2", &created_at).await.unwrap();

    let tag1 = repo.get_or_create_tag("rust").await.unwrap();
    let tag2 = repo.get_or_create_tag("web").await.unwrap();

    repo.add_tag_to_message(msg1, tag1.id).await.unwrap();
    repo.add_tag_to_message(msg2, tag1.id).await.unwrap();
    repo.add_tag_to_message(msg2, tag2.id).await.unwrap();

    let tags = repo.get_tags_with_count().await.unwrap();

    // 找到 rust 标签
    let rust_tag = tags.iter().find(|t| t.name == "rust").unwrap();
    assert_eq!(rust_tag.count, 2);

    // 找到 web 标签
    let web_tag = tags.iter().find(|t| t.name == "web").unwrap();
    assert_eq!(web_tag.count, 1);
}

/// 测试创建回复
#[actix_rt::test]
async fn test_create_reply() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let msg_id = repo.create_message("Original message", &created_at)
        .await
        .unwrap();

    let reply_id = repo.create_reply(msg_id, "This is a reply", &created_at)
        .await
        .unwrap();

    assert!(reply_id > 0);

    let total = repo.get_total_replies().await.unwrap();
    assert_eq!(total, 1);
}

/// 测试创建多条回复
#[actix_rt::test]
async fn test_create_multiple_replies() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let msg_id = repo.create_message("Original message", &created_at)
        .await
        .unwrap();

    for i in 1..=5 {
        repo.create_reply(msg_id, &format!("Reply {}", i), &created_at)
            .await
            .unwrap();
    }

    let total = repo.get_total_replies().await.unwrap();
    assert_eq!(total, 5);
}

/// 测试删除回复
#[actix_rt::test]
async fn test_delete_reply() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let msg_id = repo.create_message("Original message", &created_at)
        .await
        .unwrap();
    let reply_id = repo.create_reply(msg_id, "Reply to delete", &created_at)
        .await
        .unwrap();

    // 确认创建成功
    let total = repo.get_total_replies().await.unwrap();
    assert_eq!(total, 1);

    // 删除回复
    repo.delete_reply(reply_id).await.unwrap();

    // 确认删除成功
    let total = repo.get_total_replies().await.unwrap();
    assert_eq!(total, 0);
}

/// 测试统计功能
#[actix_rt::test]
async fn test_stats_operations() {
    let repo = create_test_repo().await;

    // 初始值应该是 0
    let value = repo.get_stat("test_stat").await.unwrap();
    assert_eq!(value, 0);

    // 增加统计
    repo.increment_stat("test_stat").await.unwrap();
    let value = repo.get_stat("test_stat").await.unwrap();
    assert_eq!(value, 1);

    // 再次增加
    repo.increment_stat("test_stat").await.unwrap();
    let value = repo.get_stat("test_stat").await.unwrap();
    assert_eq!(value, 2);
}

/// 测试每日统计
#[actix_rt::test]
async fn test_daily_stats() {
    let repo = create_test_repo().await;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // 更新留言统计
    repo.update_daily_stats(&today, true).await.unwrap();

    let stats = repo.get_daily_stats().await.unwrap();
    assert!(!stats.is_empty());
    assert_eq!(stats[0].message_count, 1);
    assert_eq!(stats[0].reply_count, 0);

    // 更新回复统计
    repo.update_daily_stats(&today, false).await.unwrap();

    let stats = repo.get_daily_stats().await.unwrap();
    assert_eq!(stats[0].message_count, 1);
    assert_eq!(stats[0].reply_count, 1);
}

/// 测试清理旧留言
#[actix_rt::test]
async fn test_cleanup_old_messages() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建 15 条留言
    for i in 1..=15 {
        repo.create_message(&format!("Message {}", i), &created_at)
            .await
            .unwrap();
    }

    // 验证创建了 15 条
    let count = repo.count_messages().await.unwrap();
    assert_eq!(count, 15);

    // 清理，只保留 10 条
    repo.cleanup_old_messages(10).await.unwrap();

    // 验证只剩 10 条
    let count = repo.count_messages().await.unwrap();
    assert_eq!(count, 10);
}

/// 测试获取留言及其标签（批量）
#[actix_rt::test]
async fn test_get_messages_with_tags_batch() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建留言并添加标签
    let msg1 = repo.create_message("Message 1", &created_at).await.unwrap();
    let msg2 = repo.create_message("Message 2", &created_at).await.unwrap();

    let tag1 = repo.get_or_create_tag("rust").await.unwrap();
    let tag2 = repo.get_or_create_tag("web").await.unwrap();

    repo.add_tag_to_message(msg1, tag1.id).await.unwrap();
    repo.add_tag_to_message(msg2, tag2.id).await.unwrap();

    let messages = repo.get_messages_with_tags_batch(1, 10).await.unwrap();

    assert_eq!(messages.len(), 2);

    // 验证每个留言都有正确的标签
    for msg in &messages {
        if msg.id == msg1 {
            assert_eq!(msg.tags.len(), 1);
            assert_eq!(msg.tags[0].name, "rust");
        } else if msg.id == msg2 {
            assert_eq!(msg.tags.len(), 1);
            assert_eq!(msg.tags[0].name, "web");
        }
    }
}

/// 测试获取留言的回复（批量）
#[actix_rt::test]
async fn test_get_replies_for_messages_batch() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建留言和回复
    let msg1 = repo.create_message("Message 1", &created_at).await.unwrap();
    let msg2 = repo.create_message("Message 2", &created_at).await.unwrap();

    repo.create_reply(msg1, "Reply 1 to msg1", &created_at)
        .await
        .unwrap();
    repo.create_reply(msg1, "Reply 2 to msg1", &created_at)
        .await
        .unwrap();
    repo.create_reply(msg2, "Reply 1 to msg2", &created_at)
        .await
        .unwrap();

    let replies = repo.get_replies_for_messages_batch(&[msg1, msg2]).await.unwrap();

    assert_eq!(replies.get(&msg1).unwrap().len(), 2);
    assert_eq!(replies.get(&msg2).unwrap().len(), 1);
}

/// 测试获取指定标签的留言
#[actix_rt::test]
async fn test_get_messages_by_tag() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建留言并添加标签
    let msg1 = repo.create_message("Rust message", &created_at).await.unwrap();
    let msg2 = repo.create_message("Web message", &created_at).await.unwrap();
    let msg3 = repo.create_message("Another Rust message", &created_at)
        .await
        .unwrap();

    let rust_tag = repo.get_or_create_tag("rust").await.unwrap();
    let web_tag = repo.get_or_create_tag("web").await.unwrap();

    repo.add_tag_to_message(msg1, rust_tag.id).await.unwrap();
    repo.add_tag_to_message(msg2, web_tag.id).await.unwrap();
    repo.add_tag_to_message(msg3, rust_tag.id).await.unwrap();

    // 按 rust 标签筛选
    let count = repo.count_messages_by_tag(rust_tag.id).await.unwrap();
    assert_eq!(count, 2);

    let messages = repo.get_messages_by_tag_with_tags_batch(rust_tag.id, 1, 10)
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);

    // 按 web 标签筛选
    let count = repo.count_messages_by_tag(web_tag.id).await.unwrap();
    assert_eq!(count, 1);
}

/// 测试 API: get_messages_since
#[actix_rt::test]
async fn test_get_messages_since() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建多条留言
    let msg1 = repo.create_message("Message 1", &created_at).await.unwrap();
    let msg2 = repo.create_message("Message 2", &created_at).await.unwrap();
    let _msg3 = repo.create_message("Message 3", &created_at).await.unwrap();

    // 获取 id > msg1 的留言
    let messages = repo.get_messages_since(msg1, 10).await.unwrap();
    assert_eq!(messages.len(), 2);

    // 获取 id > msg2 的留言
    let messages = repo.get_messages_since(msg2, 10).await.unwrap();
    assert_eq!(messages.len(), 1);

    // 测试 limit
    let messages = repo.get_messages_since(0, 2).await.unwrap();
    assert_eq!(messages.len(), 2);
}

/// 测试获取平均留言长度
#[actix_rt::test]
async fn test_get_average_message_length() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 空数据库应该返回 0
    let avg = repo.get_average_message_length().await.unwrap();
    assert_eq!(avg, 0.0);

    // 创建留言
    repo.create_message("12345", &created_at).await.unwrap(); // 5 字符
    repo.create_message("1234567890", &created_at).await.unwrap(); // 10 字符

    let avg = repo.get_average_message_length().await.unwrap();
    assert_eq!(avg, 7.5);
}

/// 测试获取时段分布
#[actix_rt::test]
async fn test_get_hourly_distribution() {
    let repo = create_test_repo().await;

    // 空数据库应该返回全 0
    let hourly = repo.get_hourly_distribution().await.unwrap();
    assert_eq!(hourly.len(), 24);
    assert!(hourly.iter().all(|&h| h == 0));

    // 创建一些留言
    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    repo.create_message("Message 1", &created_at).await.unwrap();
    repo.create_message("Message 2", &created_at).await.unwrap();

    let hourly = repo.get_hourly_distribution().await.unwrap();
    // 至少有一个时段有留言
    assert!(hourly.iter().any(|&h| h > 0));
}

/// 测试获取热门留言
#[actix_rt::test]
async fn test_get_top_messages_by_replies() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建留言和回复
    let msg1 = repo.create_message("Popular message", &created_at).await.unwrap();
    let msg2 = repo.create_message("Less popular", &created_at).await.unwrap();

    // msg1 有 3 条回复
    repo.create_reply(msg1, "Reply 1", &created_at).await.unwrap();
    repo.create_reply(msg1, "Reply 2", &created_at).await.unwrap();
    repo.create_reply(msg1, "Reply 3", &created_at).await.unwrap();

    // msg2 有 1 条回复
    repo.create_reply(msg2, "Reply 1", &created_at).await.unwrap();

    let top = repo.get_top_messages_by_replies(5).await.unwrap();
    assert_eq!(top.len(), 2);

    // 第一条应该是最多回复的
    assert_eq!(top[0].0, "Popular message");
    assert_eq!(top[0].1, 3);
}

/// 测试删除留言时级联删除回复
#[actix_rt::test]
async fn test_cascade_delete_replies() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建留言和回复
    let msg_id = repo.create_message("Message with replies", &created_at)
        .await
        .unwrap();
    repo.create_reply(msg_id, "Reply 1", &created_at).await.unwrap();
    repo.create_reply(msg_id, "Reply 2", &created_at).await.unwrap();

    // 确认回复存在
    let total = repo.get_total_replies().await.unwrap();
    assert_eq!(total, 2);

    // 删除留言
    repo.delete_message(msg_id).await.unwrap();

    // 回复应该被级联删除
    let total = repo.get_total_replies().await.unwrap();
    assert_eq!(total, 0);
}

/// 测试删除留言时级联删除标签关联
#[actix_rt::test]
async fn test_cascade_delete_tags() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建留言和标签
    let msg_id = repo.create_message("Message with tags", &created_at)
        .await
        .unwrap();
    let tag = repo.get_or_create_tag("test").await.unwrap();
    repo.add_tag_to_message(msg_id, tag.id).await.unwrap();

    // 确认标签关联存在
    let tags = repo.get_tags_with_count().await.unwrap();
    assert_eq!(tags[0].count, 1);

    // 删除留言
    repo.delete_message(msg_id).await.unwrap();

    // 标签应该还在，但计数应该为 0
    let tags = repo.get_tags_with_count().await.unwrap();
    assert_eq!(tags[0].count, 0);
}

/// 测试搜索留言并返回标签的批量查询
#[actix_rt::test]
async fn test_search_messages_with_tags_batch() {
    let repo = create_test_repo().await;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // 创建留言并添加标签
    let msg1 = repo.create_message("Rust programming is fun", &created_at).await.unwrap();
    let msg2 = repo.create_message("Web development with Rust", &created_at).await.unwrap();
    let msg3 = repo.create_message("Python is also great", &created_at).await.unwrap();

    let rust_tag = repo.get_or_create_tag("rust").await.unwrap();
    let web_tag = repo.get_or_create_tag("web").await.unwrap();
    let python_tag = repo.get_or_create_tag("python").await.unwrap();

    repo.add_tag_to_message(msg1, rust_tag.id).await.unwrap();
    repo.add_tag_to_message(msg2, rust_tag.id).await.unwrap();
    repo.add_tag_to_message(msg2, web_tag.id).await.unwrap();
    repo.add_tag_to_message(msg3, python_tag.id).await.unwrap();

    // 搜索包含 "Rust" 的留言
    let messages = repo.search_messages_with_tags_batch("Rust", 1, 10).await.unwrap();

    assert_eq!(messages.len(), 2);

    // 验证每个留言都有正确的标签
    for msg in &messages {
        assert!(msg.content.contains("Rust"));
        if msg.id == msg1 {
            assert_eq!(msg.tags.len(), 1);
            assert_eq!(msg.tags[0].name, "rust");
        } else if msg.id == msg2 {
            assert_eq!(msg.tags.len(), 2);
            let tag_names: Vec<&str> = msg.tags.iter().map(|t| t.name.as_str()).collect();
            assert!(tag_names.contains(&"rust"));
            assert!(tag_names.contains(&"web"));
        }
    }

    // 搜索不存在的内容
    let messages = repo.search_messages_with_tags_batch("NotFound", 1, 10).await.unwrap();
    assert_eq!(messages.len(), 0);
}

/// 测试 get_tags_for_messages_batch 传入空数组
#[actix_rt::test]
async fn test_get_tags_for_messages_batch_empty_array() {
    let repo = create_test_repo().await;

    // 传入空数组应该返回空的 HashMap
    let result = repo.get_tags_for_messages_batch(&[]).await.unwrap();
    assert!(result.is_empty());
}

/// 测试 get_replies_for_messages_batch 传入空数组
#[actix_rt::test]
async fn test_get_replies_for_messages_batch_empty_array() {
    let repo = create_test_repo().await;

    // 传入空数组应该返回空的 HashMap
    let result = repo.get_replies_for_messages_batch(&[]).await.unwrap();
    assert!(result.is_empty());
}

/// 测试 generate_tag_color 的颜色一致性（通过 get_or_create_tag 间接测试）
#[actix_rt::test]
async fn test_tag_color_consistency() {
    let repo = create_test_repo().await;

    // 多次创建相同名称的标签，颜色应该一致
    let tag1 = repo.get_or_create_tag("rust").await.unwrap();
    let tag2 = repo.get_or_create_tag("rust").await.unwrap();

    // 相同名称的标签应该有相同的颜色
    assert_eq!(tag1.color, tag2.color);

    // 不同名称的标签可能有不同的颜色
    let tag3 = repo.get_or_create_tag("web").await.unwrap();
    let tag4 = repo.get_or_create_tag("api").await.unwrap();

    // 验证颜色格式正确（应该是 6 位 hex 颜色）
    assert!(tag1.color.starts_with('#'));
    assert_eq!(tag1.color.len(), 7);
    assert!(tag3.color.starts_with('#'));
    assert_eq!(tag3.color.len(), 7);
    assert!(tag4.color.starts_with('#'));
    assert_eq!(tag4.color.len(), 7);

    // 再次验证相同名称的标签颜色一致性
    let tag5 = repo.get_or_create_tag("web").await.unwrap();
    assert_eq!(tag3.color, tag5.color);
}

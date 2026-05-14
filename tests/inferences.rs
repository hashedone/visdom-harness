use serde_json::json;
use tempfile::NamedTempFile;
use visdom_harness::{
    db,
    inferences,
    llm::{InferenceMessage, InferenceResult, MessageRole, ToolCallRecord},
};

fn tempfile_db_url(f: &NamedTempFile) -> String {
    format!("sqlite://{}?mode=rwc", f.path().display())
}

#[tokio::test]
async fn inference_record_get_list_round_trip() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = tempfile_db_url(&db_file);
    let pool = db::connect_and_migrate(&db_url).await.unwrap();

    let system_prompt = "You are a helpful assistant.";
    let messages = vec![InferenceMessage {
        role: MessageRole::User,
        content: "What tools do you have?".to_string(),
    }];
    let tool_call = ToolCallRecord {
        id: "call_abc123".to_string(),
        name: "search".to_string(),
        arguments: json!({ "query": "Rust async" }),
    };
    let result = InferenceResult {
        prompt_text: "What tools do you have?".to_string(),
        response_text: "I have a search tool.".to_string(),
        tool_calls: vec![tool_call.clone()],
    };

    let record = inferences::record(&pool, result, system_prompt, &messages)
        .await
        .unwrap();

    assert_eq!(record.system_prompt, system_prompt);
    assert_eq!(record.response_text, "I have a search tool.");
    assert!(!record.id.is_empty());
    assert!(!record.created_at.is_empty());

    let tool_calls_roundtrip: Vec<ToolCallRecord> =
        serde_json::from_str(&record.tool_calls_json).unwrap();
    assert_eq!(tool_calls_roundtrip.len(), 1);
    assert_eq!(tool_calls_roundtrip[0].name, "search");
    assert_eq!(tool_calls_roundtrip[0].arguments["query"], "Rust async");

    let messages_roundtrip: Vec<InferenceMessage> =
        serde_json::from_str(&record.request_messages_json).unwrap();
    assert_eq!(messages_roundtrip.len(), 1);
    assert_eq!(messages_roundtrip[0].content, "What tools do you have?");

    let fetched = inferences::get(&pool, &record.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, record.id);
    assert_eq!(fetched.response_text, record.response_text);
    assert_eq!(fetched.tool_calls_json, record.tool_calls_json);
    assert_eq!(fetched.request_messages_json, record.request_messages_json);
    assert_eq!(fetched.system_prompt, record.system_prompt);
    assert_eq!(fetched.created_at, record.created_at);

    let listed = inferences::list(&pool, 10).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, record.id);
}

#[tokio::test]
async fn inference_get_nonexistent_returns_none() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = tempfile_db_url(&db_file);
    let pool = db::connect_and_migrate(&db_url).await.unwrap();

    let result = inferences::get(&pool, "00000000-0000-0000-0000-000000000000")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn inference_list_empty_returns_empty_vec() {
    let db_file = NamedTempFile::new().unwrap();
    let db_url = tempfile_db_url(&db_file);
    let pool = db::connect_and_migrate(&db_url).await.unwrap();

    let listed = inferences::list(&pool, 10).await.unwrap();
    assert!(listed.is_empty());
}

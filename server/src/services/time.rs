pub fn now_service() -> String {
    let now = chrono::Utc::now().to_string();
    now
}

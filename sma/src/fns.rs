use uuid::Uuid;

pub fn get_unique_id() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
}
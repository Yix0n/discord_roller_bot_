pub mod draw_alg;
pub mod special_decks;
pub mod card_selector_handler;

pub fn parse_inner_key(key: impl Into<String>, user_id: impl Into<u64>) -> String {
    let key = key.into().replace(".", "|").to_lowercase();
    let user_parsed = user_id.into().to_string();

    format!("{}.{}", user_parsed, &key)
}

pub fn deparse_inner_key(key: impl Into<String>) -> (String, u64) {
    let key = key.into();
    let parts: Vec<&str> = key.split(".").collect();

    let id = parts[0].parse::<u64>().unwrap();
    let key = parts[1].to_string();

    (key, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_parsing() {
        let mock_user_id = 483562896836529836u64;
        let mock_key = "allfallsdown";

        let key = parse_inner_key(mock_key, mock_user_id);

        assert_eq!(key, format!("{}.{}", mock_user_id, mock_key));
    }

}
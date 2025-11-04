use chrono::Utc;

pub const DATALAKE_PATH: &str = "/app/datalake";

pub fn header_body_split(text: &str) -> (String, String) {
    let start_marker = "*** START OF THE PROJECT GUTENBERG EBOOK";
    let end_marker = "*** END OF THE PROJECT GUTENBERG EBOOK";

    if let Some(start_pos) = text.find(start_marker) {
        let header = text[..start_pos].to_string();

        if let Some(end_pos) = text.find(end_marker) {
            let body_start = text[start_pos..]
                .find('\n')
                .map(|pos| start_pos + pos + 1)
                .unwrap_or(start_pos);
            let body = text[body_start..end_pos].to_string();
            return (header, body);
        }
    }

    (text.to_string(), String::new())
}

pub fn create_datalake_path(book_id: u32) -> String {
    let now = Utc::now();
    let date_str = now.format("%Y%m%d").to_string();
    let subdir = format!("{:02}", book_id % 100);
    format!("{}/{}/{}", DATALAKE_PATH, date_str, subdir)
}

use std::fs;

pub const DATALAKE_PATH: &str = "/app/datalake";

pub fn find_book_files(book_id: u32) -> Option<(String, String)> {
    if let Ok(entries) = fs::read_dir(DATALAKE_PATH) {
        for date_entry in entries.flatten() {
            if date_entry
                .file_type()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
            {
                if let Ok(subdir_entries) = fs::read_dir(date_entry.path()) {
                    for subdir_entry in subdir_entries.flatten() {
                        if subdir_entry
                            .file_type()
                            .map(|ft| ft.is_dir())
                            .unwrap_or(false)
                        {
                            let header_path =
                                subdir_entry.path().join(format!("header_{}.txt", book_id));
                            let body_path =
                                subdir_entry.path().join(format!("body_{}.txt", book_id));

                            if header_path.exists() && body_path.exists() {
                                return Some((
                                    header_path.to_string_lossy().to_string(),
                                    body_path.to_string_lossy().to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
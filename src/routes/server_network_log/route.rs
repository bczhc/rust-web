use axum::extract::Query;
use axum::response::IntoResponse;

use crate::routes::server_network_log::{
    compress_entries, search_entry_range, search_entry_single, write_entries_text, Input, LogEntry,
    Mode,
};
use crate::ResponseJson;

type ResJson = ResponseJson<LogEntry>;

pub async fn get(query: Option<Query<Input>>) -> axum::response::Response {
    let mode: Option<Mode> = try {
        let query = query.as_ref()?;
        if query.time.contains("..") {
            let mut split = query.time.split("..");
            let from = split.next()?.parse::<u64>().ok()?;
            let to = split.next()?.parse::<u64>().ok()?;
            Mode::Range(from, to)
        } else {
            let timestamp = query.time.parse::<u64>().ok()?;
            Mode::Single(timestamp)
        }
    };
    let Some(mode) = mode else {
        return ResJson::error(1, "Invalid query").into_response();
    };

    let result: anyhow::Result<axum::response::Response> = try {
        match mode {
            Mode::Single(timestamp) => {
                let entry = search_entry_single(timestamp)?;
                ResJson::ok(entry).into_response()
            }
            Mode::Range(from, to) => {
                let entries = search_entry_range(from, to)?;

                let do_compression = query.map(|x| x.bzip3.unwrap_or(true)).unwrap_or(true);
                if do_compression {
                    let Ok(data) = compress_entries(&entries) else {
                        return ResJson::error(1, "Compression failed").into_response();
                    };
                    data.into_response()
                } else {
                    let mut string = String::new();
                    write_entries_text(&entries, &mut string);
                    string.into_response()
                }
            }
        }
    };
    match result {
        Ok(r) => r,
        Err(e) => ResJson::error(1, e.to_string()).into_response(),
    }
}

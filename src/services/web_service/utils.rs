pub fn extract_query_param(uri: &str, key: &str) -> Option<String> {
    let query_start = uri.find('?')?;
    let query_string = &uri[query_start + 1..];
    
    for pair in query_string.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == key {
                if let Ok(decoded) = percent_encoding::percent_decode_str(v).decode_utf8() {
                    return Some(decoded.into_owned());
                }
            }
        }
    }
    None
}

pub fn sanitize_fat_filename(raw_name: &str) -> String {
    let decoded_name = percent_encoding::percent_decode_str(raw_name)
        .decode_utf8_lossy()
        .into_owned();

    let parts: Vec<&str> = decoded_name.split('.').collect();
    let ext = if parts.len() > 1 { parts.last().unwrap_or(&"bin") } else { &"bin" };
    let base = parts[0];

    let filtered_base: String = base.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let safe_base = if filtered_base.is_empty() { "file".to_string() } else { filtered_base.chars().take(8).collect() };
    let safe_ext: String = ext.chars().filter(|c| c.is_ascii_alphanumeric()).take(3).collect();

    format!("{}.{}", safe_base, safe_ext).to_lowercase()
}
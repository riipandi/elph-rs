pub fn oauth_success_html(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>{title}</title></head>
<body style="font-family:system-ui,sans-serif;padding:2rem;text-align:center">
<h1>{title}</h1><p>You can close this window and return to the application.</p>
</body></html>"#
    )
}

pub fn oauth_error_html(message: &str, detail: Option<&str>) -> String {
    let detail_html = detail
        .map(|d| format!("<p style=\"color:#666\">{d}</p>"))
        .unwrap_or_default();
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Authentication failed</title></head>
<body style="font-family:system-ui,sans-serif;padding:2rem;text-align:center">
<h1>Authentication failed</h1><p>{message}</p>{detail_html}
</body></html>"#
    )
}

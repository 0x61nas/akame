//
#[macro_export]
macro_rules! not_found {
    () => {
        $crate::not_found!(::std::vec![])
    };
    ($body: expr) => {
        $crate::not_found!($body, $crate::header::mime::TEXT_PLAIN_UTF_8)
    };
    (html: $body: expr) => {
        $crate::not_found!($body, $crate::header::mime::TEXT_HTML_UTF_8)
    };
    ($body: expr, $mime: expr) => {{
        let body = $body;
        $crate::Response::builder()
            .status($crate::StatusCode::NOT_FOUND)
            .header_pair($crate::header::ContentType($mime))
            .header($crate::header::CONTENT_LENGTH, body.len())
            .body(body)
    }};
}

#[macro_export]
macro_rules! resource {
    ($path: expr, $mime: expr) => {{
        if let Ok(file) = $crate::_fs::File::open($path).await {
            $crate::resource!(file: file, $mime)
        } else {
            $crate::not_found!()
        }
    }};
    (file: $file: ident, $mime: expr) => {{
        let len = $file.metadata().await?.len();
        $crate::Response::builder()
            .status($crate::StatusCode::OK)
            .header_pair($crate::header::ContentType($mime))
            .header($crate::header::CONTENT_LENGTH, len)
            .body_file($file)
    }};
}

#[cfg(feature = "serde_json")]
#[macro_export]
macro_rules! json {
    ($($json:tt)+) => {{
        let body = $crate::_json!($( $json )+).to_string();
        $crate::Response::builder()
            .status($crate::StatusCode::OK)
            .header_pair($crate::header::ContentType(
                $crate::header::mime::APPLICATION_JSON,
            ))
            .header($crate::header::CONTENT_LENGTH, body.len())
            .body(body)
    }};
}

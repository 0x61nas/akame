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
        if let Ok(file) = $crate::fs::File::open(dbg!($path)).await {
            let len = file.metadata().await?.len();
            $crate::Response::builder()
                .status($crate::StatusCode::OK)
                .header_pair($crate::header::ContentType($mime))
                .header($crate::header::CONTENT_LENGTH, len)
                .body_file(file)
        } else {
            $crate::not_found!()
        }
    }};
}

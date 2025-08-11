pub const KB: usize = 1024;
pub const PORT: usize = 3000;
pub const MAX_BODY_SIZE: usize = 100 * KB; // 1 MB
pub const MAX_HEADER_SIZE: usize = 1 * KB; // 1 KB 

// #[derive(Serialize,Deserialize, Debug)]
pub enum ContentType {
    // #[serde(rename = "application/json")]
    ApplicationJson,
    // #[serde(rename = "text/html")]
    TextHtml,
}

pub enum MethodType {
    GET,
    POST,
    PUT,
    DELETE,
}

pub struct Request {
    path: String,
    host: String,
    method: MethodType,
    content_type: ContentType,
    body: [u8; MAX_BODY_SIZE],
    headers: [u8; MAX_HEADER_SIZE],
}

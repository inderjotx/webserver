pub const KB: usize = 1024;
pub const PORT: usize = 3000;
pub const MAX_HEADER_SIZE: usize = 1 * KB; // 1 KB 

#[derive(Debug)]
pub enum ContentType {
    ApplicationJson,
    TextHtml,
    TextPlain,
    Unknown,
}

#[derive(Debug)]
pub enum MethodType {
    GET,
    POST,
    PUT,
    DELETE,
    UNKNOWN,
}

const supported_http_versions: [&str; 3] = ["HTTP/1.1", "HTTP/1.0", "HTTP/2.0"];
pub struct Request {
    pub http_v: String,
    pub path: String,
    pub host: String,
    pub method: MethodType,
    pub content_type: ContentType,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn new() -> Self {
        Request {
            path: String::new(),
            host: String::new(),
            method: MethodType::UNKNOWN,
            content_type: ContentType::Unknown,
            body: Vec::new(),
            headers: Vec::new(),
            http_v: String::new(),
        }
    }

    pub fn parse_from_bytes(data: &[u8]) -> Result<Self, String> {
        let mut request = Request::new();
        let data_str = String::from_utf8_lossy(data);
        let lines: Vec<&str> = data_str.lines().collect();

        if lines.is_empty() {
            return Err("Empty request".to_string());
        }

        // Parse request line (first line)
        let request_line: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line.len() < 3 {
            return Err("Invalid request line".to_string());
        }
        println!("Request line : {:?}", request_line);

        // Parse method
        request.method = match request_line[0] {
            "GET" => MethodType::GET,
            "POST" => MethodType::POST,
            "PUT" => MethodType::PUT,
            "DELETE" => MethodType::DELETE,
            _ => MethodType::UNKNOWN,
        };

        // Parse path
        request.path = request_line[1].to_string();
        request.http_v = request_line[2].to_string();

        if !supported_http_versions.contains(&request.http_v.as_str()) {
            return Err("Invalid http type ".to_string());
        }

        // Parse headers
        let mut body_start = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }

            if i == 0 {
                continue; // Skip request line
            }

            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();

                match key.as_str() {
                    "host" => request.host = value.clone(),
                    "content-type" => {
                        request.content_type = if value.contains("application/json") {
                            ContentType::ApplicationJson
                        } else if value.contains("text/html") {
                            ContentType::TextHtml
                        } else if value.contains("text/plain") {
                            ContentType::TextPlain
                        } else {
                            ContentType::Unknown
                        };
                    }
                    _ => {}
                }

                request.headers.push((key, value));
            }
        }

        // Parse body if present
        if body_start < lines.len() {
            let body_content = lines[body_start..].join("\n");
            request.body = body_content.into_bytes();
        }

        Ok(request)
    }
}

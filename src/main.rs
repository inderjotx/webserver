use serde::de::value::Error;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod types;
use types::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let url = format!("127.0.0.1:{PORT}");
    println!("Server listening on {}", &url);
    let listener = TcpListener::bind(url).await?;

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        tokio::spawn(async move {
            let request = match get_request(&mut socket).await {
                Ok(data) => match Request::parse_from_bytes(&data) {
                    Ok(parsed_request) => parsed_request,
                    Err(e) => {
                        eprintln!("Failed to parse request: {}", e);
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read request: {}", e);
                    return;
                }
            };

            let response = handle_request(&request.path).await;

            match response {
                Ok(content) => {
                    let response = get_header(content);
                    socket.write_all(response.as_bytes()).await.unwrap();
                }
                Err(_) => {
                    let content = handle_request("/not-found")
                        .await
                        .expect("Error handling not found request");
                    let response = get_header(content);
                    socket.write_all(response.as_bytes()).await.unwrap();
                }
            }
        });
    }
}

async fn get_request(stream: &mut TcpStream) -> std::io::Result<[u8; MAX_HEADER_SIZE]> {
    let mut buffer = [0u8; MAX_HEADER_SIZE];
    stream.read(&mut buffer).await?;
    Ok(buffer)
}

fn get_header(res: Response) -> String {
    format!(
        "HTTP/1.1 {} OK\r\n\
                 Content-Type: {}\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 \r\n\
                 {}",
        res.code,
        res.header,
        res.content.len(),
        res.content
    )
}

async fn handle_request(path: &str) -> Result<Response, std::io::Error> {
    let mut content = String::new();

    let root_paths = ["/", "/index.html", "/www/index.html", "/www"];

    if root_paths.contains(&path) {
        let mut file = File::open("./www/index.html").await?;
        file.read_to_string(&mut content).await?;
        return Ok(Response::new(200, ContentType::TextHtml, content));
    } else {
        let file_path = format!("./www{path}/index.html",);
        let mut file = File::open(file_path).await?;
        file.read_to_string(&mut content).await?;

        return Ok(Response::new(200, ContentType::TextHtml, content));
    }
}

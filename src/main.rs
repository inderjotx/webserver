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

            let content = format!(
                "Request received:\nMethod: {:?}\nPath: {}\nHost: {}",
                request.method, request.path, request.host
            );
            let response = get_header(content, 200, "application/text");

            socket.write_all(response.as_bytes()).await.unwrap();
        });
    }
}

async fn get_request(stream: &mut TcpStream) -> std::io::Result<[u8; MAX_HEADER_SIZE]> {
    let mut buffer = [0u8; MAX_HEADER_SIZE];
    stream.read(&mut buffer).await?;
    Ok(buffer)
}

fn get_header(body: String, status_code: u32, content_type: &str) -> String {
    format!(
        "HTTP/1.1 {status_code} OK\r\n\
                 Content-Type: {content_type}\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 \r\n\
                 {}",
        body.len(),
        body
    )
}

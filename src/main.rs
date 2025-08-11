use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
mod types;
use types::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let url = format!("127.0.0.1:{PORT}");
    let listner = TcpListener::bind(url).await?;

    loop {
        let (mut socket, _) = listner.accept().await?;

        tokio::spawn(async move {
            let content = String::from("Hello World");
            let response = get_header(content);

            let request = get_request(&mut socket).await.unwrap();
            println!("{}", String::from_utf8_lossy(&request));

            socket.write_all(response.as_bytes()).await.unwrap();
        });
    }
}

async fn get_request(steam: &mut TcpStream) -> std::io::Result<[u8; MAX_HEADER_SIZE]> {
    let mut buffer = [0u8; MAX_HEADER_SIZE];
    steam.read(&mut buffer).await?;
    Ok(buffer)
}

fn get_header(body: String) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
                 Content-Type: text/plain\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\
                 \r\n\
                 {}",
        body.len(),
        body
    )
}

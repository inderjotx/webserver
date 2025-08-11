use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

const PORT: usize = 3000;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let url = format!("127.0.0.1:{PORT}");
    let listner = TcpListener::bind(url).await?;

    loop {
        let (mut socket, _) = listner.accept().await?;

        tokio::spawn(async move {
            let content = String::from("Hello World");
            let response = get_header(content);

            socket.write_all(response.as_bytes()).await.unwrap();
        });
    }
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

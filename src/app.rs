use std::sync::Arc;

use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
mod types;
use types::*;

pub struct App {
    listner: Arc<TcpListener>,
    queue: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>,
    abort_signal: Arc<Mutex<bool>>,
}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

impl App {
    pub async fn init(port: u32) -> Result<Self> {
        let url = format!("127.0.0.1:{port}");
        Ok(App {
            listner: Arc::new(TcpListener::bind(url).await?),
            queue: Arc::new(Mutex::new(vec![])),
            abort_signal: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        // create two spawn one for receiving request and one for handling
        let mut set = JoinSet::new();

        println!("Preparing to listen");

        let listener = Arc::clone(&self.listner);
        let queue = Arc::clone(&self.queue);
        let signal = Arc::clone(&self.abort_signal);

        set.spawn(async move { req_listen(listener, queue, signal).await });

        println!("Listening Setup Done");

        let queue = Arc::clone(&self.queue);
        let signal = Arc::clone(&self.abort_signal);

        println!("Prepare client handling...");

        set.spawn(async move { handle(queue, signal).await });

        println!("Client handling setup Done");

        let stop_signal = Arc::clone(&self.abort_signal);

        tokio::spawn(async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                eprintln!("Failed to listen for Ctrl+C: {}", e);
                return;
            }
            println!("Shutting server down...");
            let mut lock = stop_signal.lock().await;
            *lock = true;
        });

        set.join_next().await;
        set.join_next().await;

        Ok(())
    }
}

async fn req_listen(
    listener: Arc<TcpListener>,
    queue: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>,
    signal: Arc<Mutex<bool>>,
) {
    let duration = Duration::from_secs(1);
    loop {
        if let Ok(data) = tokio::time::timeout(duration, listener.accept()).await {
            match data {
                Ok((socket, addr)) => {
                    let cloned_queue = Arc::clone(&queue);
                    tokio::spawn(async move {
                        let mut queue = cloned_queue.lock().await;
                        queue.push((socket, addr));
                    });
                }
                _ => {}
            }
        }

        let lock = signal.lock().await;
        if *lock {
            break;
        }
    }

    println!("Stoped Listenning !")
}

async fn handle(queue: Arc<Mutex<Vec<(TcpStream, SocketAddr)>>>, signal: Arc<Mutex<bool>>) {
    loop {
        let mut set = JoinSet::new();

        loop {
            let mut lock = queue.lock().await;

            if lock.is_empty() {
                drop(lock);
                tokio::time::sleep(Duration::from_millis(50)).await;
            } else if let Some((mut socket, _)) = lock.pop() {
                set.spawn(async move {
                    let _ = tokio::time::timeout(Duration::from_secs(5), req_handler(&mut socket))
                        .await;
                });
            } else {
                drop(lock)
            }

            let lock = signal.lock().await;
            if *lock {
                break;
            }
        }

        set.abort_all();
        println!("Stoped handling clients");
        break;
    }
}

async fn req_handler(stream: &mut TcpStream) -> Result<()> {
    let req = req_buffer(stream).await?;

    let parsed_req = Request::parse_from_bytes(&req)?;
    let res = req_map(&parsed_req.path).await?;
    let formatted_res = res_formatter(res);
    stream.write(formatted_res.as_bytes()).await?;
    Ok(())
}

async fn req_buffer(stream: &mut TcpStream) -> std::io::Result<[u8; MAX_HEADER_SIZE]> {
    let mut buffer = [0u8; MAX_HEADER_SIZE];
    stream.read(&mut buffer).await?;
    Ok(buffer)
}

fn res_formatter(res: Response) -> String {
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

async fn req_map(path: &str) -> Result<Response> {
    let mut content = String::new();

    let root_paths = ["/", "/index.html", "/www/index.html", "/www"];

    if root_paths.contains(&path) {
        let mut file = File::open("./www/index.html").await?;
        file.read_to_string(&mut content).await?;

        return Ok(Response::new(200, ContentType::TextHtml, content));
    } else {
        let file_path = format!("./www{path}/index.html",);
        let fallback_path = format!("./www{path}");
        let not_found_path = format!("./www/not-found/index.html");

        let mut file = match File::open(file_path).await {
            Ok(f) => f,
            Err(_) => match File::open(fallback_path).await {
                Ok(f) => f,
                Err(_) => File::open(not_found_path).await?,
            },
        };

        file.read_to_string(&mut content).await?;

        return Ok(Response::new(200, ContentType::TextHtml, content));
    }
}

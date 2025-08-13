mod app;
use app::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::init(3000).await?;
    app.start().await?;
    Ok(())
}

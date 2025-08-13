mod app;
use app::*;

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::init(3000).await.expect("Error initlizing app");

    app.start().await.expect("Error in app instance");
    Ok(())
}

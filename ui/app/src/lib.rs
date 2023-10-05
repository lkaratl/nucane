use tracing::info;

use ui_config::CONFIG;
use ui_core::Ui;

pub async fn run() {
    info!("▶ ui running...");
    let ui = Ui::new();
    ui_rest_api_server::run(CONFIG.application.port, ui).await;
}

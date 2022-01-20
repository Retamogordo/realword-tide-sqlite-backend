use once_cell::sync::OnceCell;

use realworld_tide_sqlite_backend::*;

static APP: OnceCell<app::App> = OnceCell::new();

#[async_std::main]
async fn main() -> std::result::Result<(), crate::errors::BackendError> {
    let app = app::App::new();

    APP.set(app).expect("Cannot create application instance.");
    
    APP.get().unwrap().run().await
}

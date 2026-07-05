use tower_http::services::ServeDir;

use crate::config::Config;

pub fn pages_router() -> ServeDir {
    let config = Config::get();
    let pages_path = &config.pages.path;
    ServeDir::new(pages_path)
        .append_index_html_on_directories(true)
}

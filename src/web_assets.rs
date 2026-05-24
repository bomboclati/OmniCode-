use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/web_ui/"]
pub struct WebAssets;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/web_ui/"]
struct Assets;

fn main() {
    println!("cargo:rerun-if-changed=src/web_ui/");
}

use anyhow::Result;

pub struct Tunnel {
    pub url: String,
    pub is_active: bool,
}

impl Tunnel {
    pub async fn create(port: u16) -> Result<Self> {
        let url = format!("https://omnicode-{}.tunnel.dev", uuid::Uuid::new_v4());
        println!("Creating tunnel to port {}", port);
        println!("Tunnel URL: {}", url);

        Ok(Self {
            url,
            is_active: true,
        })
    }

    pub async fn close(&mut self) -> Result<()> {
        self.is_active = false;
        println!("Tunnel closed");
        Ok(())
    }
}

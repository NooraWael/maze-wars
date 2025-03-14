#[derive(Debug)]
pub struct Server {
    host: String,
    port: u16,
    min_players: u8,
    max_players: u8,
}

impl Server {
    pub fn new<S: Into<String>>(host: S, port: u16) -> Server {
        Server {
            host: host.into(),
            port,
            min_players: 1,
            max_players: 10,
        }
    }

    pub fn min_players(&mut self, min: u8) -> &mut Self {
        self.min_players = min;
        self
    }

    pub fn max_players(&mut self, max: u8) -> &mut Self {
        self.max_players = max;
        self
    }

    pub fn start(&mut self) -> &mut Self {
        println!("Starting server at {}:{}", self.host, self.port);
        self
    }
}

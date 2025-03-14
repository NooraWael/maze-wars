mod server;

use server::Server;

fn main() {
    Server::new("0.0.0.0", 2025)
        .min_players(1)
        .max_players(10)
        .start();
}

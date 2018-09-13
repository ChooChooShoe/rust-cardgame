use game::core::{self, Event};
use game::Game;
use net::server::shandle::ServerHandle;
use net::settings::ServerConfig;
use net::NetworkMode;
use std::net::ToSocketAddrs;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use ws::Sender as WsSender;
use ws::{Builder, Factory};

pub fn listen<A: ToSocketAddrs>(ip: A) {
    let settings = ServerConfig::from_disk().into();
    let (send, recv) = channel();
    let thread_handle = thread::spawn(move || core::run(recv, NetworkMode::Server, Game::new(2)));

    let factory = ServerFactory {
        sender: send,
        max_players: 2,
        next_player_id: 0,
    };
    let ws = Builder::new()
        .with_settings(settings)
        .build(factory)
        .unwrap();

    ws.listen(ip).unwrap();
    info!("Waiting for server game thread to close.");
    thread_handle.join().unwrap();
}
struct ServerFactory {
    sender: TSender<Event>,
    max_players: usize,
    next_player_id: usize,
}
impl Factory for ServerFactory {
    type Handler = ServerHandle;

    fn connection_made(&mut self, out: WsSender) -> ServerHandle {
        let id = self.next_player_id;
        self.next_player_id += 1;

        let role = if self.next_player_id > self.max_players {
            Role::GameFull
        } else if self.next_player_id == self.max_players {
            Role::Player(true) // true if final player to connect.
        } else {
            Role::Player(false)
        };

        ServerHandle::new(out, self.sender.clone(), id, role)
    }
    // fn connection_lost(&mut self, handle: ServerHandle) {
    //     warn!("Connection lost for pid {}", handle.player_id());
    //     self.sender
    //         .send(Event::CloseConnection(handle.player_id()))
    //         .unwrap_or(());
    // }
    // fn on_shutdown(&mut self) {
    //     info!("ServerFactory received WebSocket shutdown request.");
    //     self.sender.send(Event::StopAndExit()).unwrap_or(());
    // }
}

pub enum Role {
    Player(bool),
    Spectator,
    GameFull,
}

use crate::game::core::{self, Event};
use crate::game::Game;
use crate::server::{ServerHandle,ServerConfig};
use crate::net::NetworkMode;
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
        active_connections: 0,
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
    active_connections: usize,
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

        self.active_connections += 1;
        ServerHandle::new(out, self.sender.clone(), id, role)
    }
    fn connection_lost(&mut self, handle: ServerHandle) {
        info!("Connection #{} lost.", handle.player_id);
        self.active_connections -= 1;
        // The last connecction will shutdown the server.
        if self.active_connections == 0 {
            info!("All connections lost: Begin shutdown.");
            if handle.ws.shutdown().is_err() {
                warn!("Unable to send shutdown. Have we stopped already?")
            }
        }
    }
    fn on_shutdown(&mut self) {
        info!("ServerFactory received WebSocket shutdown request.");
        if self.sender.send(Event::StopAndExit()).is_ok() {
            info!("Sending 'StopAndExit' to core.")
        }
    }
}

pub enum Role {
    Player(bool),
    Spectator,
    GameFull,
}

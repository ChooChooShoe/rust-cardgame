use net::server::shandle::ServerHandle;
use entity::CardPool;
use game::core::{self, Event};
use game::Game;
use game::{Action, ActionError, OkCode};
use net::settings::ServerConfig;
use net::{Command,NetworkMode};
use player::controller::WsNetController;
use std::error::Error as StdError;
use std::net::ToSocketAddrs;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use ws::util::Timeout;
use ws::util::Token;
use ws::Sender as WsSender;
use ws::{
    Builder, CloseCode, Error, ErrorKind, Factory, Frame, Handler, Handshake, Message, Request,
    Response, Result,
};

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
    max_players: i32,
    next_player_id: i32,
}
impl Factory for ServerFactory {
    type Handler = ServerHandle;

    fn connection_made(&mut self, out: WsSender) -> ServerHandle {
        if self.next_player_id < self.max_players {
            let ret =
                ServerHandle::new(out, self.sender.clone(), self.next_player_id, Role::Player);
            self.next_player_id += 1;
            ret
        } else {
            ServerHandle::new(
                out,
                self.sender.clone(),
                self.next_player_id,
                Role::GameFull,
            )
        }
    }
    fn connection_lost(&mut self, handle: ServerHandle) {
        warn!("Connection lost for pid {}", handle.player_id());
        self.sender.send(Event::ConnectionLost(handle.player_id() as usize)).unwrap_or(());
    }
    fn on_shutdown(&mut self) {
        info!("ServerFactory received WebSocket shutdown request.");
        self.sender.send(Event::OnShutdown()).unwrap_or(());
    }
}

pub enum Role {
    Player,
    Spectator,
    GameFull,
}
#![feature(deadline_api)]
#![allow(dead_code)]
//#![allow(unused_variables)]
#![allow(unused_imports)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod config;
mod entity;
mod game;
mod net;
mod utils;
mod client;
mod server;

use crate::config::{Config, IoConfig};
use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::env;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    logger_init(LevelFilter::Info).unwrap();

    info!("Card Game Engine");
    info!("MOTD: {}", config::active().motd);
    // info!("VAlue: {}", std::mem::size_of::<serde_json::Value>());
    // info!("Val: {}", std::mem::size_of::<entity::TagVal>());
    // info!("Val: {}", std::mem::size_of::<Option<entity::TagVal>>());
    // info!("Val: {}", std::mem::size_of::<u64>());
    // info!("Number: {}", std::mem::size_of::<serde_json::Number>());

    //let (c,s) = net::create_local_clientserver();
    //game::game_loop::run(pool, board);
    let mut client = false;
    let mut max_players = 2;
    for argument in env::args() {
        info!("Args: {}", argument);
        if argument == "client" {
            client = true;
        }
    }
    // config::set_active(Config::load_from_disk());

    if client {
        client::connect("ws://127.0.0.1:3012", 0, max_players);
    } else {
        let handels = (
            mk_thread("ws_server", move || server::listen("127.0.0.1:3012", 7, max_players)),
            mk_thread("ws_client_0", move || {
                thread::sleep(Duration::from_millis(10));
                client::connect("ws://127.0.0.1:3012", 0, max_players)
            }),
             mk_thread("ws_client_1", move || {
                thread::sleep(Duration::from_millis(30));
                client::connect("ws://127.0.0.1:3012", 1, max_players)
            }),
        );
        handels.0.join().unwrap();
        handels.1.join().unwrap();
        handels.2.join().unwrap();
    }
    info!("Program exit.");
    utils::Input::flush();
}

fn mk_thread<F, T>(name: &str, f: F) -> thread::JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let builder = thread::Builder::new().name(name.into());
    builder.spawn(f).expect("Thread create error")
}

struct SimpleLogger {
    level: LevelFilter,
    start: Instant,
}

fn logger_init(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(SimpleLogger {
        level,
        start: Instant::now(),
    })).map(|()| log::set_max_level(level))
}
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let t = Instant::now().duration_since(self.start);
            //match (record.file(), record.line()) {
            match (record.target(), record.line()) {
                // //(Some(file), Some(line)) => println!(
                // (target, Some(line)) => println!(
                //     "{:03}.{:03} {}:{} [{}] {}",
                //     t.as_secs(),
                //     t.subsec_millis(),
                //     target,
                //     line,
                //     record.level().to_string(),
                //     record.args()
                // ),
                (_, _) => println!(
                    "{:03}.{:03} [{}] [{}] {}",
                    t.as_secs(),
                    t.subsec_millis(),
                    record.level().to_string(),
                    thread::current().name().unwrap_or("unnamed"),
                    record.args()
                ),
            }
        }
    }

    fn flush(&self) {}
}

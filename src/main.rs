#![feature(deadline_api)]
#![allow(dead_code)]
//#![allow(unused_variables)]
#![allow(unused_imports)]
#![feature(rust_2018_preview)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

extern crate bincode;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate ws;

mod config;
mod entity;
mod game;
mod net;
mod utils;
mod vecmap;

use crate::config::{Config, IoConfig};
use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::env;
use std::io;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    logger_init(LevelFilter::Info).unwrap();

    info!("Card Game Engine");
    // info!("VAlue: {}", std::mem::size_of::<serde_json::Value>());
    // info!("Val: {}", std::mem::size_of::<entity::TagVal>());
    // info!("Val: {}", std::mem::size_of::<Option<entity::TagVal>>());
    // info!("Val: {}", std::mem::size_of::<u64>());
    // info!("Number: {}", std::mem::size_of::<serde_json::Number>());

    //let (c,s) = net::create_local_clientserver();
    //game::game_loop::run(pool, board);
    let mut client = false;
    for argument in env::args() {
        info!("Args: {}", argument);
        if argument == "client" {
            client = true;
        }
    }
    config::set_active(Config::load_from_disk());

    if client {
        //net::client::connect("ws://127.0.0.1:3012", game);
    } else {
        let handels = (
            thread::spawn(move || net::server::listen("127.0.0.1:3012")),
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(10));
                net::client::connect("ws://127.0.0.1:3012")
            }),
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(30));
                net::client::connect("ws://127.0.0.1:3012")
            }),
        );
        handels.0.join().unwrap();
        handels.1.join().unwrap();
        handels.2.join().unwrap();
    }
    info!("Program exit.");
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
                    "{:03}.{:03} [{}] {}",
                    t.as_secs(),
                    t.subsec_millis(),
                    record.level().to_string(),
                    record.args()
                ),
            }
        }
    }

    fn flush(&self) {}
}

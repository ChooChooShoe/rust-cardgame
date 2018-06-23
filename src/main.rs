//#![feature(plugin, use_extern_macros)]
//#![plugin(tarpc_plugins)]
#![feature(deadline_api)]
//#![allow(dead_code)]
//#![allow(unused_variables)]
//#![allow(unused_imports)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
//#[macro_use]
//extern crate tarpc;

extern crate ws;
extern crate bincode;
extern crate url;
extern crate serde;
extern crate serde_json;
//extern crate fluent;
extern crate rand;
//extern crate futures;
//extern crate tokio_core;

mod entity;
mod game;
mod net;
mod player;
mod utils;
mod vecmap;

use player::Controller;
use log::{Level,Metadata,Record};
use entity::card::{Card};
use entity::CardPool;
use game::Player;
use std::collections::HashMap;
use std::time::{Instant,Duration};
use std::io;
use std::io::ErrorKind;
use std::fs::File;
use std::env;
use std::thread;

fn main() {
    log::set_logger(&SIMPLE_LOGGER).unwrap();
    log::set_max_level(SIMPLE_LOGGER.level.to_level_filter());
    
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

    if client {
        //net::client::connect("ws://127.0.0.1:3012", game);
    } else {
        let handels = (
            thread::spawn(move || {
                net::server::listen("127.0.0.1:3012")
            }),
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(10));
                net::client::connect("ws://127.0.0.1:3012")
            }),
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(30));
                net::client::connect("ws://127.0.0.1:3012")
            }),
        );
        info!("Joining Thread 0 for Server");
        handels.0.join().unwrap();
        info!("Joining Thread 1 for Client A");
        handels.1.join().unwrap();
        info!("Joining Thread 2 for Client B");
        handels.2.join().unwrap();
        
    }
    info!("Program exit.");
}

static SIMPLE_LOGGER: SimpleLogger = SimpleLogger {level: Level::Info};

struct SimpleLogger {
    level: Level
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if record.level() == Level::Info {
                println!("{}",record.args());
            }
            else {
                println!(
                "[{}] {}",
                //Utc::now(),
                //record.target(),
                record.level().to_string(),
                //.module_path().unwrap_or_default(),
                record.args());
            }
            //println!(
            //    "{}:{} [{}] {}",
            //    record.file().unwrap_or_default(),
            //    record.line().unwrap_or_default(),
            //    record.level().to_string(),
            //    record.args()
            //);
        }
    }

    fn flush(&self) {}
}
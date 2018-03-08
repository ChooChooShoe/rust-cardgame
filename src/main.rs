//#![feature(plugin, use_extern_macros)]
//#![plugin(tarpc_plugins)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
//#[macro_use]
//extern crate tarpc;

extern crate ws;
extern crate url;
extern crate bincode;
extern crate serde;
extern crate serde_json;
extern crate fluent;
extern crate rand;
//extern crate futures;
//extern crate tokio_core;

mod entity;
mod game;
mod net;
mod player;
mod utils;

use player::Controller;
use log::{Level,Metadata,Record};
use entity::card::{TagKey,TagVal,Card,CardPool};
use player::Player;
use std::collections::HashMap;
use std::time::{Instant,Duration};
use std::io;
use std::io::ErrorKind;
use std::fs::File;
use std::env;

fn main() {
    log::set_logger(&SIMPLE_LOGGER).unwrap();
    log::set_max_level(SIMPLE_LOGGER.level.to_level_filter());
    
    info!("Card Game Engine");

    let mut pool = entity::card::CardPool::new();

    for i in 0..20
    {
        
    let mut c = Card::new(10000 + i, &format!("Card #{:03}", i));
    c.insert_tag(TagKey::Attack, TagVal::Int(7 + i as i32));
    c.insert_tag(TagKey::Health, TagVal::Int(9 + i as i32));
    c.insert_tag(TagKey::Cost, TagVal::Float(3.5 * i as f32));
    c.insert_tag(TagKey::Damage, TagVal::Bool(true));
    pool.all_cards.insert(format!("auto_gen_card_{:03}", i), c);
    }

    write_test(&pool).expect("Unable to write to card database");
    read_test().expect("Unable to load card database");

    let player1 = Player::new(1,String::from("player #1"));
    let player2 = Player::new(2,String::from("player #2"));
    let mut board = game::GameBoard::new(42, player1, player2);

    //let (c,s) = net::create_local_clientserver();
    //game::game_loop::run(pool, board);
    let mut client = false;
    for argument in env::args() {
        println!("Args: {}", argument);
        if argument == "client" {
            client = true;
        }
    }

    if client {
        //net::gameserver::create_client();
        net::ws_client::connect("ws://127.0.0.1:3012");
    } else {
        //net::gameserver::create_server();
        net::ws_server::listen("127.0.0.1:3012");
    }
    println!("Program exit.");
}

fn write_test(card_collection: &CardPool) -> io::Result<()>
{
    match std::fs::create_dir("./output/"){
        Ok(()) => {info!("Created 'output' directory.")},
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists {
                info!("The 'output' directory already exists.");
                return Ok(());
            } else {
                warn!("Could not create 'output' directory. Error: {}", e)
            }
        }
    }
    let writer = File::create("./output/cards_out_pretty.json")?;
    let writer2 = File::create("./output/cards_out.json")?;

    let t = Instant::now();
    serde_json::to_writer_pretty(writer, &card_collection.all_cards)?;
    println!("Time: {:?}", t.elapsed()); 

    let t2 = Instant::now();
    serde_json::to_writer(writer2, &card_collection.all_cards)?;
    println!("Time: {:?}", t2.elapsed()); 

    Ok(())
}
fn read_test() -> io::Result<()>
{
    let rdr = File::open("./output/cards_out_pretty.json")?;
    let rdr2 = File::open("./output/cards_out.json")?;

    let t = Instant::now();
    let _card_collection2 : HashMap<String,Card> = serde_json::from_reader(rdr)?;
    println!("Time: {:?}", t.elapsed()); 

    let t2 = Instant::now();
    let _card_collection3 : HashMap<String,Card> = serde_json::from_reader(rdr2)?;
    println!("Time: {:?}", t2.elapsed());

    Ok(())
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

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate bitflags;

extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate log;
extern crate l20n;

mod card;

fn main() {
    
    println!("Card Game Engine");

    println!("Sizeof: Card {}",std::mem::size_of::<card::Card>());
    println!("Sizeof: TagKey {}",std::mem::size_of::<card::TagKey>());
    println!("Sizeof: TagVal {}",std::mem::size_of::<card::TagVal>());
    println!("Sizeof: i32 {}",std::mem::size_of::<i32>());
    println!("Sizeof: f32 {}",std::mem::size_of::<f32>());
    println!("Sizeof: f64 {}",std::mem::size_of::<f64>());
    println!("Sizeof: bool {}",std::mem::size_of::<bool>());

    let mut card_collection = Vec::new();

    for i in (0..100)
    {
        
    let mut c = card::Card::new(i, "ok");
    c.tags().insert(TagKey::Attack, TagVal::Int(7 + i as i32));
    c.tags().insert(TagKey::Health, TagVal::Int(9 + i as i32));
    c.tags().insert(TagKey::Cost, TagVal::Float(3.5 * i as f32));
    c.tags().insert(TagKey::Damage, TagVal::Bool(true));
    card_collection.push(c);
    }

    write_test(&card_collection).expect("Unable to write to card database");
    read_test().expect("Unable to load card database");


}

fn write_test(card_collection: &Vec<Card>) -> io::Result<()>

{
    std::fs::create_dir("./output/");
    let mut writer = File::create("./output/cards_out_pretty.json")?;
    let mut writer2 = File::create("./output/cards_out.json")?;

    let t = Instant::now();
    serde_json::to_writer_pretty(writer, card_collection)?;
    println!("Time: {:?}", t.elapsed()); 

    let t2 = Instant::now();
    serde_json::to_writer(writer2, card_collection)?;
    println!("Time: {:?}", t2.elapsed()); 

    Ok(())
}
fn read_test() -> io::Result<()>

{
    let mut rdr = File::open("./output/cards_out_pretty.json")?;
    let mut rdr2 = File::open("./output/cards_out.json")?;

    let t = Instant::now();
    let card_collection2 : Vec<Card> = serde_json::from_reader(rdr)?;
    println!("Time: {:?}", t.elapsed()); 

    let t2 = Instant::now();
    let card_collection3 : Vec<Card> = serde_json::from_reader(rdr2)?;
    println!("Time: {:?}", t2.elapsed());

    Ok(())
}

use card::{TagKey,TagVal,Card};
use std::collections::HashMap;
use std::time::{Instant,Duration};
use std::io;
use std::fs::File;
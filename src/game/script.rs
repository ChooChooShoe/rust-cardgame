use std::collections::HashMap;
use std::fmt;

lazy_static! {
    static ref INSTANCE: HashMap<&'static str, Box<GameScript>> = {
        let mut m: HashMap<&'static str, Box<GameScript>> = HashMap::new();
        m.insert("0", Box::new(ExampleScript {num: 43}));
        m.insert("1", Box::new(ExampleScript2 {name: String::from("ten") }));
        m.insert("banana", Box::new(ExampleScript {num: 0}));
        m
    };
}
pub struct ScriptManager;

impl ScriptManager {
    pub fn get(script_id: &str) -> &Box<GameScript> {
       INSTANCE.get(script_id).unwrap()
    }
}

pub trait GameScript: Send + Sync {
    // any event has happened.
    fn on_event(&mut self);
    // Create a new version of the script and return it in a box.
    fn box_clone(&self) -> Box<GameScript>;
}
struct ExampleScript {
    pub num: u64
}
struct ExampleScript2 {
    pub name: String
}
impl GameScript for ExampleScript {
    fn on_event(&mut self) {
        println!("ok script #{}", self.num)
    }
    fn box_clone(&self) -> Box<GameScript> {
        Box::new(ExampleScript {num: self.num})
    }
}
impl GameScript for ExampleScript2 {
    fn on_event(&mut self) {
        println!("ok script named '{}'", self.name)
    }
    fn box_clone(&self) -> Box<GameScript> {
        Box::new(ExampleScript2 {name: self.name.clone()})
    }
}

impl GameScript for () {
    fn on_event(&mut self) {
        info!("ok script empty");
    }
    fn box_clone(&self) -> Box<GameScript> {
        Box::new(())
    }
}

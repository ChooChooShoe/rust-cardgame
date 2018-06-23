use std::collections::HashMap;
use std::fmt;

lazy_static! {
    static ref INSTANCE: HashMap<&'static str, Script> = {
        let mut m: HashMap<&'static str, Script> = HashMap::new();
        m.insert("none", Script::None);
        m.insert("0", Script::new(ExampleScript { num: 43 }));
        m.insert("1", Script::new(ExampleScript2 { name: String::from("ten") }));
        m.insert("banana", Script::new(ExampleScript { num: 0 }));
        m
    };
}
pub struct ScriptManager;

impl ScriptManager {
    pub fn get(script_id: &str) -> &'static Script {
        match INSTANCE.get(script_id) {
            Some(s) => s,
            None => &Script::None,
        }
    }
}
pub enum Script {
    None,
    Boxed(Box<GameScript>),
}
impl Script {
    pub fn new<T: 'static + GameScript>(s: T) -> Script {
        Script::Boxed(Box::new(s))
    }
}
impl Clone for Script {
    fn clone(&self) -> Script {
        match self {
            Script::None => Script::None,
            Script::Boxed(x) => Script::Boxed(x.box_clone()),
        }
    }
}

pub trait GameScript: Send + Sync {
    // any event has happened.
    fn on_event(&self);
    // Create a new version of the script and return it in a box.
    fn box_clone(&self) -> Box<GameScript>;
}

impl GameScript for Script {
    fn on_event(&self) {
        match self {
            Script::None => (),
            Script::Boxed(x) => x.on_event(),
        }
    }
    fn box_clone(&self) -> Box<GameScript> {
        match self {
            Script::None => unreachable!(),
            Script::Boxed(x) => x.box_clone(),
        }
    }
}

struct ExampleScript {
    pub num: u64,
}
struct ExampleScript2 {
    pub name: String,
}
impl GameScript for ExampleScript {
    fn on_event(&self) {
        println!("ok script #{}", self.num)
    }
    fn box_clone(&self) -> Box<GameScript> {
        Box::new(ExampleScript { num: self.num })
    }
}
impl GameScript for ExampleScript2 {
    fn on_event(&self) {
        println!("ok script named '{}'", self.name)
    }
    fn box_clone(&self) -> Box<GameScript> {
        Box::new(ExampleScript2 {
            name: self.name.clone(),
        })
    }
}

impl GameScript for () {
    fn on_event(&self) {
        info!("ok script empty");
    }
    fn box_clone(&self) -> Box<GameScript> {
        Box::new(())
    }
}

use std::collections::HashMap;
use std::fmt;

lazy_static! {
    static ref INSTANCE: HashMap<&'static str, Script> = {
        let mut m: HashMap<&'static str, Script> = HashMap::new();
        m.insert("none", Box::new(()));
        m.insert("0", ExampleScript { num: 43 }.box_clone() );
        m.insert("ten", Box::new(ExampleScript2 { name: String::from("ten") }) );
        m.insert("banana", Box::new(ExampleScript { num: 0 }));
        m
    };
}
pub struct ScriptManager;

impl ScriptManager {
    /// Test to see if there is a script with given script_id.
    pub fn is_valid(script_id: &str) -> bool {
        INSTANCE.contains_key(script_id)
    }
    /// Gets a Boxed GameScript for the given script_id.
    /// If it's an ivalid id, a default script is returned.
    pub fn get(script_id: &str) -> Box<dyn GameScript> {
        match INSTANCE.get(script_id) {
            Some(s) => s.box_clone(),
            None => Box::new(()),
        }
    }
}
pub type Script = Box<dyn GameScript>;

pub trait GameScript: Send + Sync {
    /// Any event has happened. WIP
    fn on_event(&self);
    /// Create a new version of the script and return it in a box.
    fn box_clone(&self) -> Box<GameScript>;
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

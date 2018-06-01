pub enum Trigger {
    OnCardDrawn(usize),
    AfterCardDrawn(),
    OnCardCantBeDrawn(),
    OnTurnStart(),
    OnTurnEnd(),
    OnBetweenTurns(),
}

impl Trigger {
    pub fn cancelable(&self) -> bool {
        true
    }
}
pub struct Dispatch {
    trigger_callbacks: Vec<Box<Fn(&mut Trigger) + Send>>
}
impl Dispatch {
    pub fn new() -> Dispatch {
        Dispatch {
            trigger_callbacks: Vec::new()
        }
    }

    pub fn register_event<F: 'static + Send>(&mut self, callback: F)
    where
        F: Fn(&mut Trigger) -> (),
    {
        self.trigger_callbacks.push(Box::new(callback))
    }
}


pub trait Watcher {
    fn notify(&mut self, trigger: &mut Trigger);
}

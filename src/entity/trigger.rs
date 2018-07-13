use entity::Card;
use game::Player;
use std::collections::HashMap;
use game::script::GameScript;
use std::fmt;

pub enum Trigger<'a> {
    OnCardDrawn(&'a mut Player, &'a mut Card),
    AfterCardDrawn(&'a mut Card),
    OnCardDrawFail(&'a mut Player),
    OnPlayCard(&'a mut Player, &'a mut Card, bool),
    OnTurnStart(),
    OnTurnEnd(),
    OnBetweenTurns(),
}

impl<'a> fmt::Debug for Trigger<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Trigger::OnCardDrawn(_,_) => write!(f, "Trigger::OnCardDrawn"),
            Trigger::AfterCardDrawn(_) => write!(f, "Trigger::OnCardDrawn"),
            Trigger::OnCardDrawFail(_) => write!(f, "Trigger::OnCardDrawFail"),
            Trigger::OnPlayCard(_,_,_) => write!(f, "Trigger::OnPlayCard"),
            Trigger::OnTurnStart() => write!(f, "Trigger::OnTurnStart"),
            Trigger::OnTurnEnd() => write!(f, "Trigger::OnTurnEnd"),
            Trigger::OnBetweenTurns() => write!(f, "Trigger::OnBetweenTurns"),
        }
    }
}

impl<'a> Trigger<'a> {
    pub fn cancelable(&self) -> bool {
        match self {
            Trigger::OnPlayCard(_, _, _) => true,
            _ => false,
        }
    }
    pub fn is_canceled(&self) -> bool {
        match self {
            Trigger::OnPlayCard(_, _, canceled) => *canceled,
            _ => false,
        }
    }
    pub fn set_canceled(&mut self, canceled: bool) {
        match self {
            Trigger::OnPlayCard(_, _, x) => *x = canceled,
            _ => (),
        }
    }
    #[inline]
    fn pre_broadcast(&mut self) {
        match self {
            //Trigger::OnCardDrawn(_player, card) => card.script().on_event(),
            //Trigger::AfterCardDrawn(card) => card.script().on_event(),
            //Trigger::OnPlayCard(_player, card, _) => card.script().on_event(),
            _ => (),
        }
    }
    #[inline]
    fn post_broadcast(&mut self) {
        match self {
            _ => (),
        }
    }
}
use std::sync::RwLock;
lazy_static! {
    static ref INSTANCE: RwLock<Dispatch> = RwLock::new(Dispatch::new());
}
pub struct Dispatch {
    trigger_callbacks: HashMap<u32, Box<Fn(&mut Trigger) + 'static + Sync + Send>>,
}
impl Dispatch {
    fn new() -> Dispatch {
        Dispatch {
            trigger_callbacks: HashMap::new(),
        }
    }

    pub fn register_event<F: 'static + Sync + Send>(key: u32, callback: F)
    where
        F: Fn(&mut Trigger) -> (),
    {
        let mut s = INSTANCE.write().unwrap();
        s.trigger_callbacks.insert(key, Box::new(callback));
    }

    pub fn remove_event(key: u32) {
        let mut s = INSTANCE.write().unwrap();
        s.trigger_callbacks.remove(&key);
    }

    pub fn broadcast(mut trigger: Trigger) {
        info!("Broadcasting {:?}", trigger);
        trigger.pre_broadcast();
        if trigger.cancelable() {
            for x in INSTANCE.read().unwrap().trigger_callbacks.iter() {
                info!("Broadcast: Trigger for {}", x.0);
                x.1(&mut trigger);
                if trigger.is_canceled() {
                    info!("Broadcast: Cancled on {}", x.0);
                    break;
                }
            }
        } else {
            for x in INSTANCE.read().unwrap().trigger_callbacks.iter() {
                info!("Broadcast: Trigger for {}", x.0);
                x.1(&mut trigger);
            }
        }
        trigger.post_broadcast();
    }
}
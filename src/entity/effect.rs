// An Effect represents any card text, passive or active effect that changes any part of the game
pub struct Effect {
    name: String,
    active: bool,
    persistant: bool,
}

impl Effect {
    pub fn new(name: &str, persistant: bool) -> Effect {
        Effect {
            name: String::from(name),
            active: true,
            persistant,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn set_active(&mut self, active: bool) {
        self.active = active
    }
}

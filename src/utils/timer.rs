use std::time::*;
use std::thread;

pub struct Timer {
    start_time: Instant,
    end_time: Instant,
}
impl Timer {
    pub fn from_duration(duration: Duration) -> Timer {
        let now = Instant::now();
        Timer {
            start_time: now,
            end_time: now + duration,
        }
    }
    pub fn is_out_of_time(&self) -> bool {
        Instant::now() > self.end_time
    }
    pub fn time_left(&self) -> Duration {
        let now = Instant::now();
        if now > self.end_time {
            Duration::new(0, 0)
        } else {
            self.end_time - now
        }
    }
    pub fn wait(&self) {
        thread::sleep(self.time_left())
    }
}
impl Default for Timer {
    fn default() -> Self {
        Timer::from_duration(Duration::new(0, 0))
    }
}

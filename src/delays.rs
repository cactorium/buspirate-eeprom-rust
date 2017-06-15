pub use std::thread::sleep;

use std::time::Duration;

lazy_static! {
    pub static ref RESP_DELAY_REALLY_SHORT: Duration = Duration::from_millis(1);
    pub static ref RESP_DELAY_SHORT: Duration = Duration::from_millis(2);
    pub static ref RESP_DELAY_LONG: Duration = Duration::from_millis(100);
    pub static ref RESP_DELAY_REALLY_LONG: Duration = Duration::from_millis(500);
}



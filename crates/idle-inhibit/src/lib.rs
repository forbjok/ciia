#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
use self::unix as platform;
#[cfg(windows)]
use self::windows as platform;

use platform::InnerInhibitor;

pub struct Inhibitor {
    _inner: InnerInhibitor,
}

pub mod command;

use std::{
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use tracing::error;

const SLEEP_DURATION: Duration = Duration::from_secs(1);

pub trait Poker: 'static + Send {
    fn poke(&mut self) -> anyhow::Result<()>;
}

/// Inhibitor that works by "poking" some sort of service periodically
/// to let it know that the system is not idle.
pub struct PokeInhibitor<P>
where
    P: Poker,
{
    cancel: Arc<AtomicBool>,
    join_handle: Option<thread::JoinHandle<()>>,
    _poker: PhantomData<P>,
}

impl<P> PokeInhibitor<P>
where
    P: Poker,
{
    pub fn new(poker: P, frequency: Duration) -> anyhow::Result<Self> {
        let cancel = Arc::new(AtomicBool::new(false));

        let join_handle = {
            let mut poker = poker;
            let cancel = cancel.clone();

            let mut last_poke: Option<Instant> = None;

            Some(thread::spawn(move || {
                loop {
                    if cancel.load(Ordering::Acquire) {
                        break;
                    }

                    thread::sleep(SLEEP_DURATION);

                    let now = Instant::now();
                    let time_since_last = last_poke.map(|v| now - v).unwrap_or_else(|| Duration::MAX);
                    if time_since_last < frequency {
                        continue;
                    }

                    if let Err(err) = poker.poke() {
                        error!("Poke error: {}", err);
                        break;
                    }

                    last_poke = Some(now);
                }
            }))
        };

        Ok(Self {
            cancel,
            join_handle,
            _poker: Default::default(),
        })
    }
}

impl<P> Drop for PokeInhibitor<P>
where
    P: Poker,
{
    fn drop(&mut self) {
        let Some(join_handle) = self.join_handle.take() else {
            return;
        };

        self.cancel.store(true, Ordering::Release);

        _ = join_handle.join();
    }
}

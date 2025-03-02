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

pub trait PokeImpl {
    fn poke() -> anyhow::Result<()>;
}

pub struct PokeInhibitor<PI>
where
    PI: PokeImpl,
{
    cancel: Arc<AtomicBool>,
    join_handle: Option<thread::JoinHandle<()>>,
    _poke_impl: PhantomData<PI>,
}

impl<PI> PokeInhibitor<PI>
where
    PI: PokeImpl,
{
    pub fn new(frequency: Duration) -> anyhow::Result<Self> {
        let cancel = Arc::new(AtomicBool::new(false));

        let join_handle = {
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

                    if let Err(err) = PI::poke() {
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
            _poke_impl: Default::default(),
        })
    }
}

impl<PI> Drop for PokeInhibitor<PI>
where
    PI: PokeImpl,
{
    fn drop(&mut self) {
        let Some(join_handle) = self.join_handle.take() else {
            return;
        };

        self.cancel.store(true, Ordering::Release);

        _ = join_handle.join();
    }
}

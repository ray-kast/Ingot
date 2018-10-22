use std::{
  collections::VecDeque,
  sync::{Arc, Barrier, Mutex},
  thread::{self, JoinHandle},
};

struct Worker {
  handle: JoinHandle<()>,
}

pub struct OneshotPool<T>
where
  T: Send + 'static,
{
  q: Arc<Mutex<VecDeque<T>>>,
  workers: Vec<Worker>,
}

impl<T> OneshotPool<T>
where
  T: Send + 'static,
{
  pub fn new<Q, I, C, F, D>(tasks: Q, closures: I, f: F, done: D) -> Self
  where
    Q: IntoIterator<Item = T>,
    I: IntoIterator<Item = C>,
    C: Send + 'static,
    F: Fn(usize, &C, T) -> () + Clone + Send + 'static,
    D: FnOnce() -> () + Send + 'static,
  {
    let q = Arc::new(Mutex::new(tasks.into_iter().collect::<VecDeque<_>>()));

    let closures: Vec<_> = closures.into_iter().collect();

    let nclosures = closures.len();

    let start_bar = Arc::new(Barrier::new(nclosures + 1));
    let end_bar = Arc::new(Barrier::new(nclosures));

    let mut done = Some(done);

    let workers: Vec<_> = closures
      .into_iter()
      .enumerate()
      .map(|(id, closure)| {
        let start_bar = start_bar.clone();
        let q = q.clone();
        let f = f.clone();
        let end_bar = end_bar.clone();

        let done = done.take();

        let handle = thread::spawn(move || {
          start_bar.wait();

          loop {
            match {
              let mut q = q.lock().unwrap();
              q.pop_front()
            } {
              Some(t) => f(id, &closure, t),
              None => break,
            }
          }

          end_bar.wait();

          done.map(|d| d());
        });

        Worker { handle }
      })
      .collect();

    start_bar.wait();

    Self { q, workers }
  }

  pub fn abort(self) {
    self.q.lock().unwrap().clear();

    self.join()
  }

  pub fn join(self) {
    for worker in self.workers {
      worker.handle.join().unwrap();
    }
  }
}

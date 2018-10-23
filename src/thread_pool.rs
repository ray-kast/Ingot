use std::{
  collections::HashSet,
  sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
  },
  thread::{self, JoinHandle},
};

pub struct ThreadPool<T>
where
  T: Send + 'static,
{
  task_tx: Sender<T>, // TODO: maybe replace this with a Deque or something?
  sched_tx: Sender<SchedMessage>,
  sched_handle: JoinHandle<()>,
}

enum SchedMessage {
  Sleep(usize),
  WakeAll,
  ForceWakeAll,
  Stop,
}

enum WorkerMessage {
  Continue,
  Stop,
}

use self::{SchedMessage as SM, WorkerMessage as WM};

impl<T> ThreadPool<T>
where
  T: Send + 'static,
{
  pub fn new<I, C, F>(closures: I, f: F) -> Self
  where
    I: IntoIterator<Item = C>,
    C: Send + 'static,
    F: Fn(usize, &C, T) -> () + Clone + Send + 'static,
  {
    let (task_tx, task_rx) = channel();

    let task_rx = Arc::new(Mutex::new(task_rx));

    let (sched_tx, sched_rx) = channel();

    let workers: Vec<_> = closures
      .into_iter()
      .enumerate()
      .map(|(id, closure)| {
        let (tx, rx) = channel::<WM>();

        let task_rx = task_rx.clone();
        let sched_tx = sched_tx.clone();
        let f = f.clone();

        let handle = thread::spawn(move || loop {
          match rx.recv().expect("failed to receive WM") {
            WM::Continue => loop {
              match {
                let task_rx = task_rx.lock().unwrap();
                task_rx.try_recv()
              } {
                Ok(t) => f(id, &closure, t),
                Err(_) => break,
              }
            },
            WM::Stop => break,
          }

          sched_tx
            .send(SM::Sleep(id))
            .expect("failed to send SM::Sleep");
        });

        (handle, tx)
      })
      .collect();

    let sched_handle = thread::spawn(move || {
      let mut awake: HashSet<usize> = HashSet::new();
      let mut stopping = false;

      loop {
        match sched_rx.recv().expect("failed to receive SM") {
          SM::Sleep(i) => {
            awake.remove(&i);

            if awake.len() == 0 && stopping {
              for (_, tx) in &workers {
                tx.send(WM::Stop).expect("failed to send WM::Stop");
              }

              for (handle, _) in workers {
                handle.join().expect("a worker died");
              }

              break;
            }
          },
          SM::WakeAll => {
            for i in 0..workers.len() {
              if awake.insert(i) {
                workers[i]
                  .1
                  .send(WM::Continue)
                  .expect("failed to send WM::Continue");
              }
            }
          },
          SM::ForceWakeAll => {
            for i in 0..workers.len() {
              workers[i]
                .1
                .send(WM::Continue)
                .expect("failed to force-send WM::Continue");
            }
          },
          SM::Stop => stopping = true,
        }
      }
    });

    Self {
      task_tx,
      sched_tx,
      sched_handle,
    }
  }

  // TODO: make this return a Result maybe?
  pub fn queue(&self, task: T) {
    self.task_tx.send(task).expect("failed to send task");
    self
      .sched_tx
      .send(SM::WakeAll)
      .expect("failed to send SM::WakeAll");
  }

  pub fn join(self) {
    self
      .sched_tx
      .send(SM::Stop)
      .expect("failed to send SM::Stop");

    // NB: I'm not just using SM::WakeAll because if a worker panics, it will
    //     never send its WM::Sleep message and will never be awoken, causing
    //     ThreadPool::join to hang
    self
      .sched_tx
      .send(SM::ForceWakeAll)
      .expect("failed to send SM::ForceWakeAll");
    self.sched_handle.join().expect("scheduler died");
  }
}

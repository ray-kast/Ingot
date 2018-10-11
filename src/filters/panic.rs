use super::prelude::*;

struct Proc;

#[allow(dead_code)]
pub struct PanicFilter {
  proc: Arc<Proc>,
}

impl PanicFilter {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      proc: Arc::new(Proc),
    }
  }
}

impl Filter for PanicFilter {
  fn name(&self) -> &str {
    "PANIC"
  }

  fn params(&self) {}

  fn proc(&self) -> ArcProc {
    self.proc.clone() as ArcProc
  }
}

impl RenderProc for Proc {
  fn process_tile(&self, _: Arc<Tile>) {
    panic!("debug panic");
  }
}

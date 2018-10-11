use super::prelude::*;
use render::DummyRenderProc;

pub struct DummyFilter {
  proc: Arc<DummyRenderProc>,
}

impl DummyFilter {
  pub fn new() -> Self {
    Self {
      proc: Arc::new(DummyRenderProc),
    }
  }
}

impl Filter for DummyFilter {
  fn name(&self) -> &str {
    "None"
  }

  fn params(&self) {}

  fn proc(&self) -> ArcProc {
    self.proc.clone() as ArcProc
  }
}

use super::prelude::*;
use render::DummyRenderProc;

pub struct DummyFilter {
  params: Vec<Param>,
  proc: Arc<DummyRenderProc>,
}

impl DummyFilter {
  pub fn new() -> Self {
    Self {
      params: Vec::new(),
      proc: Arc::new(DummyRenderProc),
    }
  }
}

impl Filter for DummyFilter {
  fn name(&self) -> &str { "None" }

  fn params(&self) -> &Vec<Param> { &self.params }

  fn proc(&self) -> ArcProc { self.proc.clone() as ArcProc }
}

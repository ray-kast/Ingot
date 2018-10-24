use super::prelude::*;

struct Proc;

pub struct BlankFilter {
  params: Vec<Param>,
  proc: Arc<Proc>,
}

impl BlankFilter {
  pub fn new() -> Self {
    Self {
      params: Vec::new(),
      proc: Arc::new(Proc),
    }
  }
}

impl Filter for BlankFilter {
  fn name(&self) -> &str { "Blank" }

  fn params(&self) -> &Vec<Param> { &self.params }

  fn proc(&self) -> ArcProc { self.proc.clone() as ArcProc }
}

impl RenderProc for Proc {
  fn process_tile(&self, tile: &Tile, _: &CancelTok) {
    let mut out_buf = tile.out_buf();

    for i in 0..out_buf.len() {
      out_buf[i] = Pixel::new(0.0, 0.5, 0.0, 1.0);
    }
  }
}

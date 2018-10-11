use super::prelude::*;

struct Proc;

pub struct InvertFilter {
  proc: Arc<Proc>,
}

impl InvertFilter {
  pub fn new() -> Self {
    InvertFilter {
      proc: Arc::new(Proc),
    }
  }
}

impl Filter for InvertFilter {
  fn name(&self) -> &str {
    "Invert"
  }

  fn params(&self) {}

  fn proc(&self) -> ArcProc {
    self.proc.clone() as ArcProc
  }
}

impl RenderProc for Proc {
  fn process_tile(&self, tile: Arc<Tile>) {
    let mut out_buf = tile.out_buf();

    for r in 0..tile.h() {
      let r_stride = r * tile.w();

      for c in 0..tile.w() {
        let px = tile.get_input(c, r);

        out_buf[(r_stride + c) as usize] =
          Pixel::new(1.0 - px[0], 1.0 - px[1], 1.0 - px[2], px[3]);
      }
    }
  }
}

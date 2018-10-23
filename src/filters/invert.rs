use super::prelude::*;

struct Proc {
  param_amt: Arc<RangedParam<f64>>,
}

pub struct InvertFilter {
  params: Vec<Param>,
  proc: Arc<Proc>,
}

impl InvertFilter {
  pub fn new() -> Self {
    let param_amt = Arc::new(RangedParam::new(1.0, 0.0, 1.0, 0.0, 1.0));

    Self {
      params: vec![Param("Amount".to_string(), param_amt.clone().into())],
      proc: Arc::new(Proc { param_amt }),
    }
  }
}

impl Filter for InvertFilter {
  fn name(&self) -> &str { "Invert" }

  fn params(&self) -> &Vec<Param> { &self.params }

  fn proc(&self) -> ArcProc { self.proc.clone() as ArcProc }
}

impl RenderProc for Proc {
  // This is fast enough that we can ignore the cancellation token
  fn process_tile(&self, tile: &Tile, _: &CancelTok) {
    let mut out_buf = tile.out_buf();

    let amt = self.param_amt.get() as f32;

    for r in 0..tile.h() {
      let r_stride = r * tile.w();

      for c in 0..tile.w() {
        let px = tile.get_input(c, r);

        let flipped = Pixel::new(1.0 - px[0], 1.0 - px[1], 1.0 - px[2], px[3]);

        out_buf[(r_stride + c) as usize] = flipped * amt + px * (1.0 - amt);
      }
    }
  }
}

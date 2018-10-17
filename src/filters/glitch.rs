use super::prelude::*;
use rand::{self, prelude::*};
use std::{thread, time::Duration};

struct Proc {
  param_seed: Arc<IntParam>,
}

pub struct GlitchFilter {
  params: Vec<Param>,
  proc: Arc<Proc>,
}

impl GlitchFilter {
  pub fn new() -> Self {
    let param_seed = Arc::new(IntParam::new(0));

    Self {
      params: vec![Param("Seed".to_string(), param_seed.clone().into())],
      proc: Arc::new(Proc { param_seed }),
    }
  }
}

impl Filter for GlitchFilter {
  fn name(&self) -> &str {
    "Glitch"
  }

  fn params(&self) -> &Vec<Param> {
    &self.params
  }

  fn proc(&self) -> ArcProc {
    self.proc.clone() as ArcProc
  }
}

impl RenderProc for Proc {
  fn process_tile(&self, tile: &Tile) {
    let mut out_buf = tile.out_buf();

    for i in 0..out_buf.len() {
      out_buf[i] = Pixel::new(0.0, 0.5, 0.0, 1.0);
    }

    thread::sleep(Duration::from_millis(
      rand::thread_rng().gen_range(400, 700),
    ));
  }
}

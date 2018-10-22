use super::prelude::*;

struct Proc {
  param_radius: Arc<RangedParam<i32>>,
}

pub struct NaiveMedianFilter {
  params: Vec<Param>,
  proc: Arc<Proc>,
}

impl NaiveMedianFilter {
  pub fn new() -> Self {
    let param_radius = Arc::new(RangedParam::new(3, 0, 20, 0, None));

    Self {
      params: vec![
        Param("Radius".to_string(), param_radius.clone().into()),
      ],
      proc: Arc::new(Proc {
        param_radius,
      }),
    }
  }
}

impl Filter for NaiveMedianFilter {
  fn name(&self) -> &str {
    "Median Blur (naive)"
  }

  fn params(&self) -> &Vec<Param> {
    &self.params
  }

  fn proc(&self) -> ArcProc {
    self.proc.clone() as ArcProc
  }
}

impl Proc {
  fn process_px<'a>(&self, tile: &Tile, r: u32, c: u32, radius: u32) -> Pixel {
    for r2 in (r - radius)..(r + radius) {
      for c2 in (c - radius)..(c + radius) {

      }
    }


    tile.get_input(c, r)
  }
}

impl RenderProc for Proc {
  // fn begin(&self, w: u32, h: u32) {}

  fn process_tile(&self, tile: &Tile, cancel_tok: &CancelTok) {
    let mut out_buf = tile.out_buf();

    let radius = self.param_radius.get() as u32;

    'row_loop: for r in 0..tile.h() {
      let r_stride = r * tile.w();

      if cancel_tok.cancelled() {
        break 'row_loop;
      }

      for c in 0..tile.w() {
        out_buf[(r_stride + c) as usize] = self.process_px(tile, r, c, radius);
      }
    }
  }
}

use super::prelude::*;
use std::sync::RwLock;

struct Data {
  w: u32,
  h: u32,
}

struct Proc {
  data: RwLock<Data>,
  param_flipx: Arc<SwitchParam>,
  param_flipy: Arc<SwitchParam>,
}

// TODO: make the axis configurable
pub struct FlipFilter {
  params: Vec<Param>,
  proc: Arc<Proc>,
}

impl FlipFilter {
  pub fn new() -> Self {
    let param_flipx = Arc::new(SwitchParam::new(true));
    let param_flipy = Arc::new(SwitchParam::new(true));

    Self {
      params: vec![
        Param("Flip X".to_string(), param_flipx.clone().into()),
        Param("Flip Y".to_string(), param_flipy.clone().into()),
      ],
      proc: Arc::new(Proc {
        param_flipx,
        param_flipy,
        data: RwLock::new(Data { w: 0, h: 0 }),
      }),
    }
  }
}

impl Filter for FlipFilter {
  fn name(&self) -> &str {
    "Flip"
  }

  fn params(&self) -> &Vec<Param> {
    &self.params
  }

  fn proc(&self) -> ArcProc {
    self.proc.clone() as ArcProc
  }
}

impl RenderProc for Proc {
  fn begin(&self, w: u32, h: u32) {
    let mut data = self.data.write().unwrap();

    data.w = w;
    data.h = h;
  }

  fn process_tile(&self, tile: &Tile) {
    let data = self.data.read().unwrap();
    let mut out_buf = tile.out_buf();

    let flipx = self.param_flipx.get();
    let flipy = self.param_flipy.get();

    let x_axis = data.w - 1;
    let y_axis = data.h - 1;

    for r in 0..tile.h() {
      let r_stride = r * tile.w();

      for c in 0..tile.w() {
        let px = tile.global_input(
          if flipx {
            x_axis - (tile.x() + c)
          } else {
            tile.x() + c
          },
          if flipy {
            y_axis - (tile.y() + r)
          } else {
            tile.y() + r
          },
        );

        out_buf[(r_stride + c) as usize] = px;
      }
    }
  }
}

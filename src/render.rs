use image::{GenericImageView, Rgba, RgbaImage};
use nalgebra::Vector4;
use oneshot_pool::OneshotPool;
use std::{
  cmp,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
  },
};

// TODO: worry about colorspace conversions

pub type Quantum = f32;
pub type Pixel = Vector4<Quantum>;

pub struct Tile {
  x: u32,
  y: u32,
  w: u32,
  h: u32,
  in_stride: u32,
  in_buf: Arc<Vec<Pixel>>,
  out_buf: Mutex<Vec<Pixel>>,
}

impl Tile {
  pub fn x(&self) -> u32 { self.x }

  pub fn y(&self) -> u32 { self.y }

  pub fn w(&self) -> u32 { self.w }

  pub fn h(&self) -> u32 { self.h }

  pub fn get_input(&self, x: u32, y: u32) -> Pixel {
    if x >= self.w {
      panic!("x value {} out-of-bounds", x);
    }

    if y >= self.h {
      panic!("y value {} out-of-bounds", y);
    }

    self.in_buf[((self.y + y) * self.in_stride + self.x + x) as usize]
  }

  pub fn global_input(&self, x: u32, y: u32) -> Pixel {
    if x >= self.in_stride {
      panic!("x value {} out-of-bounds", x);
    }

    self.in_buf[(y * self.in_stride + x) as usize]
  }

  pub fn out_buf(&self) -> MutexGuard<Vec<Pixel>> {
    self.out_buf.lock().unwrap()
  }

  pub fn cx(&self) -> u32 { self.x + self.w / 2 }

  pub fn cy(&self) -> u32 { self.y + self.h / 2 }
}

pub struct TaggedTile<T>
where
  T: Send + Sync,
{
  tile: Tile,
  tag: T,
}

impl<T> TaggedTile<T>
where
  T: Send + Sync,
{
  pub fn tile(&self) -> &Tile { &self.tile }

  pub fn tag(&self) -> &T { &self.tag }
}

pub struct CancelTok {
  cancelled: AtomicBool,
}

impl CancelTok {
  pub fn cancelled(&self) -> bool { self.cancelled.load(Ordering::SeqCst) }
}

pub trait RenderProc {
  fn begin(&self, _w: u32, _h: u32) {}

  fn process_tile(&self, tile: &Tile, tok: &CancelTok);
}

pub trait RenderCallback {
  type Tag;

  fn before_begin(&self, _ntiles: usize) {}

  fn after_end(&self) {}

  fn abort(&self) {}

  fn handle_tile(&self, tile: Arc<TaggedTile<Self::Tag>>)
  where
    Self::Tag: Send + Sync;
}

pub struct Renderer<C>
where
  C: RenderCallback + Clone + Send + 'static,
  C::Tag: Default + Send + Sync,
{
  njobs: usize,
  w: u32,
  h: u32,
  tile_w: u32,
  tile_h: u32,
  tiles: Vec<Arc<TaggedTile<C::Tag>>>,
  worker: Option<OneshotPool<Arc<TaggedTile<C::Tag>>>>,
  proc: Arc<RenderProc + Send + Sync>,
  callback: C,
  cancel_tok: Arc<CancelTok>,
}

impl<C> Renderer<C>
where
  C: RenderCallback + Clone + Send + 'static,
  C::Tag: Default + Send + Sync,
{
  pub fn new<P>(
    tile_w: u32,
    tile_h: u32,
    njobs: usize,
    proc: Arc<P>,
    callback: C,
  ) -> Self
  where
    P: RenderProc + Send + Sync + 'static,
  {
    Self {
      njobs,
      w: 0,
      h: 0,
      tile_w,
      tile_h,
      tiles: Vec::new(),
      worker: None,
      proc,
      callback,
      cancel_tok: Arc::new(CancelTok {
        cancelled: AtomicBool::new(false),
      }),
    }
  }

  fn update_ordering(&mut self) {
    let cx = (self.w / 2) as f32;
    let cy = (self.h / 2) as f32;

    self.tiles.sort_by(|a, b| {
      let a = a.tile();
      let b = b.tile();

      let da = (((a.cx() as f32 - cx).powi(2) + (a.cy() as f32 - cy).powi(2))
        as f32)
        .sqrt();
      let db = (((b.cx() as f32 - cx).powi(2) + (b.cy() as f32 - cy).powi(2))
        as f32)
        .sqrt();

      da.partial_cmp(&db)
        .unwrap()
        .then_with(|| a.y.cmp(&b.y).then_with(|| a.x.cmp(&b.x)))
    });
  }

  fn begin_render(&mut self) {
    self.callback.before_begin(self.tiles.len());
    self.proc.begin(self.w, self.h);

    self.worker = Some(OneshotPool::new(
      self.tiles.iter().map(|t| t.clone()),
      (0..self.njobs).map(|_| {
        (
          self.proc.clone(),
          self.callback.clone(),
          self.cancel_tok.clone(),
        )
      }),
      |_id, (proc, callback, cancel_tok), tile: Arc<TaggedTile<C::Tag>>| {
        proc.process_tile(&tile.tile, &cancel_tok);

        if !cancel_tok.cancelled() {
          callback.handle_tile(tile);
        }
      },
      {
        let callback = self.callback.clone();

        move || {
          callback.after_end();
        }
      },
    ));
  }

  fn join_render(&mut self) -> bool {
    match self.worker.take() {
      Some(w) => {
        w.join();
        true
      },
      None => false,
    }
  }

  fn abort_render(&mut self) -> bool {
    self.cancel_tok.cancelled.store(true, Ordering::SeqCst);

    self.callback.abort();

    let ret = match self.worker.take() {
      Some(w) => {
        w.abort();
        true
      },
      None => false,
    };

    self.cancel_tok.cancelled.store(false, Ordering::SeqCst);

    ret
  }

  pub fn rerender(&mut self) {
    self.abort_render();
    self.begin_render();
  }

  pub fn read_input<I>(&mut self, in_img: &I)
  where
    I: GenericImageView<Pixel = Rgba<u8>>,
  {
    self.tiles.clear();

    self.abort_render();

    self.w = in_img.width();
    self.h = in_img.height();

    let tiles_x =
      self.w / self.tile_w + if self.w % self.tile_w > 0 { 1 } else { 0 };
    let tiles_y =
      self.h / self.tile_h + if self.h % self.tile_h > 0 { 1 } else { 0 };

    let in_buf = Arc::new({
      let mut in_buf = Vec::new();

      for r in 0..self.h {
        for c in 0..self.w {
          let px = in_img.get_pixel(c, r).data;

          in_buf.push(Vector4::new(
            px[0] as Quantum / 255.0,
            px[1] as Quantum / 255.0,
            px[2] as Quantum / 255.0,
            px[3] as Quantum / 255.0,
          ));
        }
      }

      in_buf
    });

    self.tiles = (0..tiles_y)
      .flat_map(|r| {
        let y = r * self.tile_h;
        let h = cmp::min(self.tile_h, self.h - y);

        (0..tiles_x).map(move |c| (h, y, c)).map(|(h, y, c)| {
          let x = c * self.tile_w;
          let w = cmp::min(self.tile_w, self.w - x);

          let out_buf: Vec<_> =
            (0..h * w).map(|_| Pixel::new(0.0, 0.0, 0.0, 0.0)).collect();

          Arc::new(TaggedTile {
            tile: Tile {
              x,
              y,
              w,
              h,
              in_stride: self.w,
              in_buf: in_buf.clone(),
              out_buf: Mutex::new(out_buf),
            },
            tag: Default::default(),
          })
        })
      })
      .collect();

    self.update_ordering();
    self.begin_render();
  }

  pub fn set_proc(&mut self, proc: Arc<RenderProc + Send + Sync>) {
    self.proc = proc;
    self.rerender();
  }

  pub fn get_output(&mut self) -> Option<RgbaImage> {
    if self.tiles.is_empty() {
      return None;
    }

    self.join_render();

    let mut img = RgbaImage::new(self.w, self.h);

    for tile in &self.tiles {
      let tile = &tile.tile;

      let buf = tile.out_buf.lock().unwrap();

      for r in 0..tile.h {
        let r_stride = r * tile.w;

        for c in 0..tile.w {
          let px = buf[(r_stride + c) as usize];

          let data = [
            (px[0] * 255.0).round() as u8,
            (px[1] * 255.0).round() as u8,
            (px[2] * 255.0).round() as u8,
            (px[3] * 255.0).round() as u8,
          ];

          img.put_pixel(tile.x + c, tile.y + r, Rgba { data });
        }
      }
    }

    return Some(img);
  }
}

impl<C> Drop for Renderer<C>
where
  C: RenderCallback + Clone + Send + 'static,
  C::Tag: Default + Send + Sync,
{
  fn drop(&mut self) { self.abort_render(); }
}

pub struct DummyRenderProc;

impl RenderProc for DummyRenderProc {
  fn process_tile(&self, tile: &Tile, _: &CancelTok) {
    let mut out_buf = tile.out_buf();

    for r in 0..tile.h {
      let r_stride = r * tile.w;

      for c in 0..tile.w {
        out_buf[(r_stride + c) as usize] = tile.get_input(c, r);
      }
    }
  }
}

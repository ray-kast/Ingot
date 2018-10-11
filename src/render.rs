use image::{GenericImageView, Rgba, RgbaImage};
use nalgebra::Vector4;
use std::{
  cmp,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
  },
};
use thread_pool::ThreadPool;

// TODO: worry about colorspace conversions

pub type Quantum = f32;
pub type Pixel = Vector4<Quantum>;

pub struct RenderTile {
  x: u32,
  y: u32,
  w: u32,
  h: u32,
  in_buf: Vec<Pixel>,
  out_buf: Mutex<Vec<Pixel>>,
  dirty: AtomicBool,
}

impl RenderTile {
  pub fn x(&self) -> u32 {
    self.x
  }
  pub fn y(&self) -> u32 {
    self.y
  }
  pub fn w(&self) -> u32 {
    self.w
  }
  pub fn h(&self) -> u32 {
    self.h
  }

  pub fn in_buf(&self) -> &Vec<Pixel> {
    &self.in_buf
  }

  pub fn out_buf(&self) -> MutexGuard<Vec<Pixel>> {
    self.out_buf.lock().unwrap()
  }

  pub fn cx(&self) -> u32 {
    self.x + self.w / 2
  }

  pub fn cy(&self) -> u32 {
    self.y + self.h / 2
  }
}

pub trait RenderProc {
  fn process_tile(&self, tile: Arc<RenderTile>);
}

pub struct Renderer<C>
where
  C: Fn(Arc<RenderTile>) -> () + Clone + Send + 'static,
{
  njobs: usize,
  w: u32,
  h: u32,
  tile_w: u32,
  tile_h: u32,
  tiles: Vec<Arc<RenderTile>>,
  worker: Option<ThreadPool<Arc<RenderTile>>>,
  proc: Arc<RenderProc + Send + Sync>,
  callback: C,
}

impl<C> Renderer<C>
where
  C: Fn(Arc<RenderTile>) -> () + Clone + Send + 'static,
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
    }
  }

  fn update_ordering(&mut self) {
    let cx = (self.w / 2) as f32;
    let cy = (self.h / 2) as f32;

    self.tiles.sort_by(|a, b| {
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
    self.worker = Some(ThreadPool::new(
      (0..self.njobs).map(|_| (self.proc.clone(), self.callback.clone())),
      |_id, (proc, callback), tile: Arc<RenderTile>| {
        // TODO: is this the right ordering?
        if tile.dirty.swap(false, Ordering::SeqCst) {
          proc.process_tile(tile.clone());

          callback(tile);
        }
      },
    ));

    let worker = self.worker.as_ref().unwrap();

    for tile in &self.tiles {
      // TODO: is this the right ordering?
      if !tile.dirty.swap(true, Ordering::SeqCst) {
        worker.queue(tile.clone());
      }
    }
  }

  fn join_render(&mut self) -> bool {
    match self.worker.take() {
      Some(w) => {
        w.join();
        true
      }
      None => false,
    }
  }

  fn abort_render(&mut self) -> bool {
    for tile in &self.tiles {
      tile.dirty.store(false, Ordering::SeqCst); // TODO: is this the right ordering?
    }

    self.join_render()
  }

  pub fn rerender(&mut self) {
    self.abort_render();
    self.begin_render();
  }

  pub fn read_input<I>(&mut self, in_img: &I)
  where
    I: GenericImageView<Pixel = Rgba<u8>>,
  {
    for tile in self.tiles.drain(0..) {
      tile.dirty.store(false, Ordering::SeqCst); // TODO: is this the right ordering?
    }

    self.join_render();

    self.w = in_img.width();
    self.h = in_img.height();

    let tiles_x =
      self.w / self.tile_w + if self.w % self.tile_w > 0 { 1 } else { 0 };
    let tiles_y =
      self.h / self.tile_h + if self.h % self.tile_h > 0 { 1 } else { 0 };

    self.tiles = (0..tiles_y)
      .flat_map(|r| {
        let y = r * self.tile_h;
        let h = cmp::min(self.tile_h, self.h - y);

        (0..tiles_x).map(move |c| (h, y, c)).map(|(h, y, c)| {
          let x = c * self.tile_w;
          let w = cmp::min(self.tile_w, self.w - x);

          let bufsize = w as usize * h as usize;

          let mut in_buf = Vec::with_capacity(bufsize);
          let mut out_buf = Vec::with_capacity(bufsize);

          let tile = in_img.view(x, y, w, h);

          for i in 0..h {
            for j in 0..w {
              let px = tile.get_pixel(j, i).data;

              in_buf.push(Vector4::new(
                px[0] as Quantum / 255.0,
                px[1] as Quantum / 255.0,
                px[2] as Quantum / 255.0,
                px[3] as Quantum / 255.0,
              ));

              out_buf.push(Vector4::new(0.0, 0.0, 0.0, 0.0));
            }
          }

          Arc::new(RenderTile {
            x,
            y,
            w,
            h,
            in_buf,
            out_buf: Mutex::new(out_buf),
            dirty: AtomicBool::new(false),
          })
        })
      })
      .collect();

    self.update_ordering();
    self.begin_render();
  }

  pub fn set_proc<P>(&mut self, proc: Arc<P>)
  where
    P: RenderProc + Send + Sync + 'static,
  {
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
  C: Fn(Arc<RenderTile>) + Clone + Send + 'static,
{
  fn drop(&mut self) {
    self.abort_render();
  }
}

pub struct DummyRenderProc;

impl RenderProc for DummyRenderProc {
  fn process_tile(&self, tile: Arc<RenderTile>) {
    let in_buf = tile.in_buf();
    let mut out_buf = tile.out_buf();

    for i in 0..in_buf.len() {
      out_buf[i] = in_buf[i];
    }
  }
}

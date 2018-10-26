use super::prelude::*;
use rand::prelude::*;
use std::{
  cmp,
  collections::{BTreeMap, Bound},
  mem,
};

struct RowData {
  radius: u32,
  offx: i32,
  offy: i32,
}

struct Data {
  w: u32,
  h: u32,
  row_data: BTreeMap<u32, RowData>,
}

struct Proc {
  data: RwLock<Data>,
  param_seed: Arc<IntParam>,
  param_perc: Arc<RangedParam<f64>>,
  param_flipat: Arc<RangedParam<f64>>,
  param_flipoff: Arc<RangedParam<f64>>,
}

pub struct GlitchFilter {
  params: Vec<Param>,
  proc: Arc<Proc>,
}

impl GlitchFilter {
  pub fn new() -> Self {
    let param_seed = Arc::new(IntParam::new(0));
    let param_perc = Arc::new(RangedParam::new(50.0, 0.0, 100.0, 0.0, 100.0));
    let param_flipat = Arc::new(RangedParam::new(0.15, 0.0, 1.0, 0.0, 1.0));
    let param_flipoff = Arc::new(RangedParam::new(1.0, 0.0, 10.0, 0.0, None));

    Self {
      params: vec![
        Param("Seed".to_string(), param_seed.clone().into()),
        Param("Percentile".to_string(), param_perc.clone().into()),
        Param("Granularity".to_string(), param_flipat.clone().into()),
        Param("Gran. Offs.".to_string(), param_flipoff.clone().into()),
      ],
      proc: Arc::new(Proc {
        data: RwLock::new(Data {
          w: 0,
          h: 0,
          row_data: BTreeMap::new(),
        }),
        param_seed,
        param_perc,
        param_flipat,
        param_flipoff,
      }),
    }
  }
}

impl Filter for GlitchFilter {
  fn name(&self) -> &str { "Glitch" }

  fn params(&self) -> &Vec<Param> { &self.params }

  fn proc(&self) -> ArcProc { self.proc.clone() as ArcProc }
}

impl Proc {
  fn process_px<'a>(
    &self,
    tile: &Tile,
    data: &RwLockReadGuard<'a, Data>,
    r: u32,
    c: u32,
    tile_data: &TileData,
    row_data: &RowData,
  ) -> Pixel {
    fn broken_quicksort(slice: &mut [Quantum], flip_len: usize) {
      fn partition(slice: &mut [Quantum], by: usize, flip: bool) -> usize {
        let mut pivot: usize = 0;

        slice.swap(0, by);

        if flip {
          for i in 1..slice.len() {
            if slice[i] > slice[pivot] {
              slice.swap(pivot, i);
              pivot = pivot + 1;
              slice.swap(pivot, i);
            }
          }
        } else {
          for i in 1..slice.len() {
            if slice[i] < slice[pivot] {
              slice.swap(pivot, i);
              pivot = pivot + 1;
              slice.swap(pivot, i);
            }
          }
        }

        pivot
      }

      if slice.len() < 20 {
        return;
      }

      let part_by = slice.len() / 2;
      let flip = slice.len() <= flip_len;
      let part_idx = partition(slice, part_by, flip); // TODO: use a better pivot

      broken_quicksort(&mut slice[0..part_idx], flip_len);
      broken_quicksort(&mut slice[part_idx + 1..], flip_len);
    }

    if row_data.radius < 1 {
      return tile.get_input(c, r);
    }

    let mut samples: [Vec<Quantum>; 4] = Default::default();

    let r = r as i32 + row_data.offy;
    let c = c as i32 + row_data.offx;
    let radius = row_data.radius as i32;

    let rx = radius;
    let ry = cmp::min(3, radius);

    for r2 in (r - ry)..(r + ry) {
      let r2 =
        cmp::max(0, cmp::min((data.h - 1) as i32, r2 + tile.y() as i32)) as u32;

      for c2 in (c - rx)..(c + rx) {
        let c2 =
          cmp::max(0, cmp::min((data.w - 1) as i32, c2 + tile.x() as i32))
            as u32;

        let px = tile.global_input(c2, r2);

        for i in 0..4 {
          samples[i].push(px[i]);
        }
      }
    }

    let mut ret = Pixel::zeros();

    for i in 0..4 {
      let vec = &mut samples[i];

      let flip_len = ((vec.len() as f64 - tile_data.flipoff)
        * tile_data.flipat)
        .round() as usize;

      broken_quicksort(vec.as_mut_slice(), flip_len);

      ret[i] = vec[((vec.len() - 1) as f64 * tile_data.perc).round() as usize];
    }

    ret
  }
}

struct TileData {
  perc: f64,
  flipat: f64,
  flipoff: f64,
}

impl RenderProc for Proc {
  fn begin(&self, w: u32, h: u32) {
    fn gen_seed(seed: u64) -> <SmallRng as SeedableRng>::Seed {
      let mut ret: <SmallRng as SeedableRng>::Seed = Default::default();

      for i in 0..ret.len() {
        let shift = (ret.len() - i - 1) * 8;

        ret[i] = if shift < 64 {
          ((seed >> shift) & 0xff) as u8
        } else {
          0
        };
      }

      ret
    }

    fn gen_rng(seeder: &mut SmallRng) -> SmallRng {
      let mut seed: <SmallRng as SeedableRng>::Seed = Default::default();

      seeder.fill(&mut seed);

      SmallRng::from_seed(seed)
    }

    let mut data = self.data.write().unwrap();

    let seed = self.param_seed.get() as u64 * w as u64 * h as u64;

    let mut seeder = SmallRng::from_seed(gen_seed(seed));

    let mut count_rng = gen_rng(&mut seeder);
    let mut radius_rng = gen_rng(&mut seeder);
    let mut offx_rng = gen_rng(&mut seeder);
    let mut offy_rng = gen_rng(&mut seeder);

    let mut row_data: Vec<(u32, RowData)> = Vec::new();
    let mut row: u32 = 0;

    while row < h {
      let count: u32 = count_rng.gen_range(10, 100);

      row = row + count;

      row_data.push((
        row,
        RowData {
          radius: radius_rng.gen_range(0, 30),
          offx: offx_rng.gen_range(-10, 10),
          offy: offy_rng.gen_range(-10, 10),
        },
      ));
    }

    data.w = w;
    data.h = h;
    data.row_data = row_data.into_iter().collect();
  }

  fn process_tile(&self, tile: &Tile, cancel_tok: &CancelTok) {
    let data = self.data.read().unwrap();
    let mut out_buf = tile.out_buf();

    let tile_data = TileData {
      perc: self.param_perc.get() / 100.0,
      flipat: 1.0 - self.param_flipat.get(),
      flipoff: self.param_flipoff.get(),
    };

    let mut row_data_src = data
      .row_data
      .range((Bound::Included(tile.y()), Bound::Unbounded));

    let mut curr_row_data: Option<&RowData> = None;
    let mut next_data_row: u32 = 0;

    'row_loop: for r in 0..tile.h() {
      let r_stride = r * tile.w();

      if r >= next_data_row {
        let (row, data) = row_data_src.next().unwrap();

        curr_row_data = Some(data);
        next_data_row = row - tile.y();
      }

      let curr_row_data = curr_row_data.unwrap();

      if curr_row_data.radius < 30 {
        if cancel_tok.cancelled() {
          break 'row_loop;
        }

        for c in 0..tile.w() {
          out_buf[(r_stride + c) as usize] =
            self.process_px(tile, &data, r, c, &tile_data, curr_row_data);
        }
      } else {
        for c in 0..tile.w() {
          if cancel_tok.cancelled() {
            break 'row_loop;
          }

          out_buf[(r_stride + c) as usize] =
            self.process_px(tile, &data, r, c, &tile_data, curr_row_data);
        }
      }
    }
  }
}

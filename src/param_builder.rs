use filters::params::*;
use gtk::{
  prelude::*, Adjustment, Align, Box as GBox, Entry as GEntry, Grid, Label, Orientation, Scale,
  SpinButton, Switch,
};
use render::{RenderCallback, Renderer};
use std::{cell::RefCell, rc::Rc};

pub fn build<C>(tool_box: &GBox, params: &Vec<Param>, renderer: &Rc<RefCell<Renderer<C>>>)
where
  C: RenderCallback + Clone + Send + 'static,
  C::Tag: Default + Send + Sync,
{
  for child in tool_box.get_children() {
    tool_box.remove(&child);
  }

  for param in params.iter() {
    build_param(tool_box, &param, renderer);
  }

  tool_box.show_all();
}

fn build_param<C>(tool_box: &GBox, param: &Param, renderer: &Rc<RefCell<Renderer<C>>>)
where
  C: RenderCallback + Clone + Send + 'static,
  C::Tag: Default + Send + Sync,
{
  use self::ParamVal as P;

  let Param(name, val) = param;

  match val {
    P::Switch(b) => {
      let switch_box = GBox::new(Orientation::Horizontal, 2);

      let label = Label::new(name.as_str());

      switch_box.pack_start(&label, true, true, 0);

      let switch = Switch::new();

      switch_box.pack_end(&switch, false, false, 0);

      tool_box.pack_start(&switch_box, false, false, 0);

      switch.set_active(b.get());

      switch.connect_state_set(autoclone!(renderer, b => move |_, val| {
        b.set(val);

        renderer.borrow_mut().rerender();

        Inhibit(false)
      }));
    }
    P::SpinInt(i) => {
      let spin_box = GBox::new(Orientation::Horizontal, 2);

      let label = Label::new(name.as_str());

      spin_box.pack_start(&label, false, false, 0);

      let spin_btn = SpinButton::new(None, 1.0, 0);

      spin_box.pack_end(&spin_btn, true, true, 0);

      tool_box.pack_start(&spin_box, false, false, 0);

      spin_btn.get_adjustment().configure(
        i.get() as f64,
        <i32>::min_value() as f64,
        <i32>::max_value() as f64,
        1.0,
        1.0,
        0.0,
      );

      spin_btn.connect_changed(autoclone!(renderer, i => move |spin_btn| {
        let val = spin_btn.get_value().round() as i32;

        if i.swap(val) == val {
          return;
        }

        renderer.borrow_mut().rerender();
      }));
    }
    P::RangedInt(r) => (),
    P::RangedFloat(r) => {
      let (scl, adj, entry) = create_ranged_numeric(tool_box, name);

      scl.set_digits(-1);

      adj.configure(r.get(), r.min(), r.max(), 0.0, 1.0, 0.0);

      scl.connect_value_changed(autoclone!(renderer, r, entry => move |scl| {
        let val = scl.get_value();

        r.set(val);

        entry.set_text(&val.to_string());

        renderer.borrow_mut().rerender();
      }));

      entry.connect_changed(autoclone!(renderer, r => move |entry| {
        let val = match entry.get_text() {
          Some(s) => s,
          None => return, // TODO: also perform proper validation
        };

        let val: f64 = match val.parse() {
          Ok(v) => v,
          Err(_) => return, // TODO: perform proper validation
        };

        if r.swap(val) == val {
          return;
        }

        renderer.borrow_mut().rerender();
      }));

      entry.connect_activate(autoclone!(renderer, r, scl => move |entry| {
        let val = match entry.get_text() {
          Some(s) => s,
          None => return, // TODO: also perform proper validation
        };

        let val: f64 = match val.parse() {
          Ok(v) => v,
          Err(_) => return, // TODO: perform proper validation
        };

        if r.swap(val) == val {
          return;
        }

        scl.set_value(r.get());

        entry.set_position(0);
        entry.select_region(0, -1);

        renderer.borrow_mut().rerender();
      }));

      entry.set_text(&r.get().to_string());
    }
  }
}

fn create_ranged_numeric(tool_box: &GBox, name: &str) -> (Scale, Adjustment, GEntry) {
  let grid = Grid::new();

  let label = Label::new(name);

  grid.attach(&label, 0, 0, 2, 1);

  let scl = Scale::new(Orientation::Horizontal, None);

  scl.set_draw_value(false);
  scl.set_hexpand(true);

  grid.attach(&scl, 0, 1, 1, 1);

  let entry = GEntry::new();

  entry.set_has_frame(false);
  entry.set_width_chars(8);

  entry.connect_focus_in_event(|entry, _| {
    entry.select_region(0, -1);
    entry.set_position(0);

    Inhibit(false)
  });

  grid.attach(&entry, 1, 1, 1, 1);

  tool_box.pack_start(&grid, false, false, 0);

  let adj = scl.get_adjustment();

  (scl, adj, entry)
}

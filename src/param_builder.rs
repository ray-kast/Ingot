use filters::params::*;
use gtk::{prelude::*, Adjustment, Box as GBox, Label, Orientation, Scale, Switch};
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
    P::Switch(s) => {
      let switch_box = GBox::new(Orientation::Horizontal, 2);

      let label = Label::new(name.as_str());

      switch_box.pack_start(&label, false, true, 0);

      let switch = Switch::new();

      switch.set_active(s.get());

      switch_box.pack_end(&switch, false, false, 0);

      tool_box.pack_start(&switch_box, false, false, 0);

      switch.connect_state_set(autoclone!(renderer, s => move |_, val| {
        s.set(val);

        renderer.borrow_mut().rerender();

        Inhibit(false)
      }));
    }
    P::RangedInt(r) => (),
    P::RangedFloat(r) => {
      let (scl, adj) = create_ranged_numeric(tool_box, renderer, name);

      scl.set_digits(-1);

      adj.configure(r.get(), r.min(), r.max(), 0.0, 1.0, 0.0);

      scl.connect_value_changed(autoclone!(renderer, r => move |scl| {
        let val = scl.get_value();

        r.set(val);

        renderer.borrow_mut().rerender();
      }));
    }
  }
}

fn create_ranged_numeric<C>(
  tool_box: &GBox,
  renderer: &Rc<RefCell<Renderer<C>>>,
  name: &str,
) -> (Scale, Adjustment)
where
  C: RenderCallback + Clone + Send + 'static,
  C::Tag: Default + Send + Sync,
{
  let label = Label::new(name);

  tool_box.pack_start(&label, false, false, 0);

  let scl = Scale::new(Orientation::Horizontal, None);

  scl.set_draw_value(false);

  tool_box.pack_start(&scl, false, false, 0);

  let adj = scl.get_adjustment();

  (scl, adj)
}

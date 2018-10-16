use filters::params::*;
use gtk::{prelude::*, Box as GBox, Label, Orientation, Switch};
use render::{RenderCallback, Renderer};
use std::{cell::RefCell, rc::Rc};

pub fn build<C>(
  tool_box: &GBox,
  params: &Vec<Param>,
  renderer: &Rc<RefCell<Renderer<C>>>,
) where
  C: RenderCallback + Clone + Send + 'static,
{
  for child in tool_box.get_children() {
    tool_box.remove(&child);
  }

  for param in params.iter() {
    build_param(tool_box, &param, renderer);
  }

  tool_box.show_all();
}

fn build_param<C>(
  tool_box: &GBox,
  param: &Param,
  renderer: &Rc<RefCell<Renderer<C>>>,
) where
  C: RenderCallback + Clone + Send + 'static,
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
    P::RangedI32(r) => (),
    P::RangedI64(r) => (),
    P::RangedF32(r) => (),
    P::RangedF64(r) => (),
  }
}

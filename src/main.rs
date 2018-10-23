#![feature(integer_atomics)]

extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate image;
extern crate nalgebra;
extern crate num_cpus;
extern crate rand;

#[macro_use]
mod autoclone;

mod app;
mod danger;
mod filters;
mod oneshot_pool;
mod param_builder;
mod render;
mod thread_pool;

use app::{flt, App};
use gio::{prelude::*, ApplicationFlags};
use gtk::Application;
use std::{cell::RefCell, env, rc::Rc};

fn main() {
  let gtk_app =
    Application::new("net.rk1024.ingot", ApplicationFlags::FLAGS_NONE).unwrap();

  let app = Rc::new(RefCell::new(None as Option<App>));

  gtk_app.connect_startup(autoclone!(app => move |gtk_app| {
    let mut app = app.borrow_mut();

    *app = Some(App::new(gtk_app, vec![
      flt(filters::FlipFilter::new()),
      flt(filters::InvertFilter::new()),
      flt(filters::NaiveMedianFilter::new()),
      flt(filters::GlitchFilter::new()),
    ]));
  }));

  gtk_app.connect_activate(|_| {});

  gtk_app.run(&env::args().collect::<Vec<_>>());
}

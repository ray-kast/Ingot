// TODO: add a README

extern crate gdk_pixbuf;
extern crate glib;
extern crate gtk;
extern crate image;
extern crate nalgebra;

mod procs;
mod render;
mod thread_pool;

macro_rules! autoclone {
  (@param _) => (_);
  (@param $x:ident) => ($x);

  (move || $body:expr) => (move || $body);
  (move |$($p:tt),+| $body:expr) => (move |$(autoclone!(@param $p),)+| $body);

  ($($n:ident),+ => move || $body:expr) => (
    {
      $(let $n = $n.clone();)+
      move || $body
    }
  );

  ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
    {
      $(let $n = $n.clone();)+
      move |$(autoclone!(@param $p),)+| $body
    }
  );
}

use gdk_pixbuf::{Colorspace, Pixbuf};
use gtk::{
  prelude::*, Builder, Button, ComboBoxText, FileChooserAction,
  FileChooserDialog, ListStore, ResponseType, Type as GType, Window,
};
use image::{DynamicImage, GenericImageView};
use render::{DummyRenderProc, RenderProc, Renderer};
use std::{
  cell::RefCell,
  collections::HashMap,
  rc::Rc,
  sync::{Arc, Mutex},
};

// NB: I'm only doing this because these types are only accessed either directly
//     on the main thread, or inside an idle callback
struct DangerPixbuf(Pixbuf);
struct DangerImage(gtk::Image);

unsafe impl Send for DangerPixbuf {}
unsafe impl Send for DangerImage {}

fn main() {
  // TODO: create a GTK Application

  gtk::init().unwrap();

  let main_glade = include_str!("res/main.glade");

  let builder = Builder::new_from_string(main_glade);

  let win =
    Rc::new(RefCell::new(builder.get_object::<Window>("_root").unwrap()));

  let image_preview = Arc::new(Mutex::new(DangerImage(
    builder.get_object::<gtk::Image>("image_preview").unwrap(),
  )));

  let in_img = Rc::new(RefCell::new(None as Option<DynamicImage>));
  let buf = Arc::new(Mutex::new(None as Option<DangerPixbuf>));

  // TODO: make these configurable
  let renderer = Rc::new(RefCell::new(Renderer::new(
    64,
    64,
    10,
    Arc::new(DummyRenderProc),
    autoclone!(image_preview, buf => move |tile| {
      glib::idle_add(autoclone!(image_preview, buf => move || {
        let out_buf = buf.lock().unwrap();

        let out_buf = match &*out_buf {
          Some(b) => &b.0,
          None => return Continue(false),
        };

        let image_preview = &image_preview.lock().unwrap().0;

        let tile_buf = tile.out_buf();

        for r in 0..tile.h() {
          let r_stride = r * tile.w();

          for c in 0..tile.w() {
            let px = tile_buf[(r_stride + c) as usize];

            out_buf.put_pixel(
              (tile.x() + c) as i32,
              (tile.y() + r) as i32,
              (px[0] * 255.0).round() as u8,
              (px[1] * 255.0).round() as u8,
              (px[2] * 255.0).round() as u8,
              (px[3] * 255.0).round() as u8
            );
          }
        }

        image_preview.set_from_pixbuf(Some(out_buf));

        Continue(false)
      }));
    }),
  )));

  let open_btn: Button = builder.get_object("open_btn").unwrap();
  let save_btn: Button = builder.get_object("save_btn").unwrap();

  let filter_select: ComboBoxText =
    builder.get_object("filter_select").unwrap();

  let filters = {
    let mut filters = HashMap::new();

    type ArcProc = Arc<RenderProc + Send + Sync>;

    for (id, name, filt) in vec![
      ("none", "None", Arc::new(render::DummyRenderProc) as ArcProc),
      (
        "invert",
        "Invert",
        Arc::new(procs::InvertRenderProc) as ArcProc,
      ),
      (
        "glitch",
        "Glitch",
        Arc::new(procs::GlitchRenderProc::new()) as ArcProc,
      ),
    ] {
      filter_select.append(id, name);
      filters.insert(id, filt);
    }

    filters
  };

  filter_select.set_active_id("none");

  {
    let win = win.borrow_mut();

    win.show(); // TODO: figure out why the startup notification has just "."

    win.connect_delete_event(|_, _| {
      gtk::main_quit();
      Inhibit(false)
    });
  }

  open_btn.connect_clicked(autoclone!(win, in_img, renderer => move |_| {
    let dlg = FileChooserDialog::new(
      Some("Open File"),
      Some(&*win.borrow_mut()),
      FileChooserAction::Open,
    );

    dlg.add_buttons(&[
      ("_Cancel", ResponseType::Cancel.into()),
      ("_Open", ResponseType::Accept.into()),
    ]);

    dlg.set_modal(true);

    match ResponseType::from(dlg.run()) {
      ResponseType::Accept => {}
      _ => {
        println!("aborting open");
        dlg.destroy();
        return;
      }
    }

    let files = dlg.get_filenames();

    dlg.destroy();

    if files.is_empty() {
      return;
    } else if files.len() > 1 {
      println!("too many files");

      return;
    }

    gtk::idle_add(autoclone!(image_preview, in_img, buf, renderer => move || {
      let mut img = in_img.borrow_mut();

      println!("loading {:?}", files[0]);

      *img = Some(match image::open(files[0].as_path()) {
        Ok(i) => i,
        Err(e) => {
          println!("  image failed to load: {:?}", e);
          return Continue(false);
        }
      });

      println!("  done");

      let mut buf = buf.lock().unwrap();

      let img = img.as_ref().unwrap();

      *buf = Some(DangerPixbuf(Pixbuf::new(
        Colorspace::Rgb,
        true,
        8,
        img.width() as i32,
        img.height() as i32
      )));

      let image_preview = &image_preview.lock().unwrap().0;
      let buf = &buf.as_ref().unwrap().0;

      image_preview.set_from_pixbuf(Some(buf));

      println!("clearing pixbuf...");

      for r in 0..img.height() {
        for c in 0..img.width() {
          buf.put_pixel(c as i32, r as i32, 0, 127, 0, 255);
        }
      }

      println!("  done");

      println!("initializing renderer...");

      renderer.borrow_mut().read_input(img);

      println!("  done");

      Continue(false)
    }));
  }));

  save_btn.connect_clicked(autoclone!(win, renderer => move |_| {
    let img = renderer.borrow_mut().get_output();

    if img.is_some() {
      let dlg = FileChooserDialog::new(
        Some("Save File"),
        Some(&*win.borrow_mut()),
        FileChooserAction::Save,
      );

      dlg.add_buttons(&[
        ("_Cancel", ResponseType::Cancel.into()),
        ("_Save", ResponseType::Accept.into()),
      ]);

      dlg.set_do_overwrite_confirmation(true);
      dlg.set_modal(true);

      match ResponseType::from(dlg.run()) {
        ResponseType::Accept => {}
        _ => {
          println!("aborting save");
          dlg.destroy();
          return;
        }
      }

      let files = dlg.get_filenames();

      dlg.destroy();

      if files.is_empty() {
        return;
      } else if files.len() > 1 {
        println!("too many files");

        return;
      }

      gtk::idle_add(move || {
        println!("saving {:?}", files[0]);

        img.as_ref().unwrap().save(files[0].clone()).unwrap();

        println!(" done");

        Continue(false)
      });
    }
  }));

  filter_select.connect_changed(autoclone!(renderer => move |el| {
    let id = match el.get_active_id() {
      Some(i) => i,
      None => return,
    };

    let filter = filters.get(&id.as_str());

    renderer.borrow_mut().set_proc(filter.unwrap().clone());
  }));

  gtk::main();
}

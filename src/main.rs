extern crate gdk_pixbuf;
extern crate gtk;
extern crate image;

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
  prelude::*, Builder, Button, FileChooserAction, FileChooserDialog,
  ResponseType, Window,
};
use image::{DynamicImage, GenericImageView};
use std::{cell::RefCell, rc::Rc};

fn main() {
  // TODO: create a GTK Application

  gtk::init().unwrap();

  let main_glade = include_str!("res/main.glade");

  let builder = Builder::new_from_string(main_glade);

  let win =
    Rc::new(RefCell::new(builder.get_object::<Window>("_root").unwrap()));

  let image_preview = Rc::new(RefCell::new(
    builder.get_object::<gtk::Image>("image_preview").unwrap(),
  ));

  let in_img = Rc::new(RefCell::new(None as Option<DynamicImage>));
  let out_img = Rc::new(RefCell::new(None as Option<DynamicImage>));
  let buf = Rc::new(RefCell::new(None as Option<Pixbuf>));

  let open_btn: Button = builder.get_object("open_btn").unwrap();
  let save_btn: Button = builder.get_object("save_btn").unwrap();

  win.borrow_mut().show();

  win.borrow_mut().connect_delete_event(|_, _| {
    gtk::main_quit();
    Inhibit(false)
  });

  open_btn.connect_clicked(autoclone!(win, in_img, out_img => move |_| {
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
    dlg.run();

    let files = dlg.get_filenames();

    dlg.destroy();

    if files.is_empty() {
      return;
    } else if files.len() > 1 {
      println!("too many files"); // TODO: make this a dialog box, maybe?

      return;
    }

    gtk::idle_add(autoclone!(in_img, out_img, buf, image_preview => move || {
      let mut img = in_img.borrow_mut();

      println!("loading {:?}", files[0]);

      *img = Some(image::open(files[0].as_path()).unwrap());

      println!("loaded {:?}", files[0]);

      let mut buf = buf.borrow_mut();

      let img = img.as_ref().unwrap();

      *buf = Some(Pixbuf::new(Colorspace::Rgb, true, 8, img.width() as i32, img.height() as i32));

      let image_preview = image_preview.borrow_mut();
      let buf = buf.as_ref().unwrap();

      image_preview.set_from_pixbuf(Some(buf));

      println!("clearing pixbuf...");

      for r in 0..img.height() {
        for c in 0..img.width() {
          buf.put_pixel(c as i32, r as i32, 0, 127, 0, 255);
        }
      }

      {
        let mut oimg = out_img.borrow_mut();

        *oimg = Some(img.clone());
      }

      println!("pixbuf cleared");

      Continue(false)
    }));
  }));

  save_btn.connect_clicked(autoclone!(win, out_img => move |_| {
    let img = out_img.borrow_mut();

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
      dlg.run();

      let files = dlg.get_filenames();

      dlg.destroy();

      if files.is_empty() {
        return;
      } else if files.len() > 1 {
        println!("too many files");

        return;
      }

      gtk::idle_add(autoclone!(out_img => move || {
        let img = out_img.borrow_mut();

        println!("saving {:?}", files[0]);

        img.as_ref().unwrap().save(files[0].clone()).unwrap();

        println!("saved {:?}", files[0]);

        Continue(false)
      }));
    }
  }));

  gtk::main();
}

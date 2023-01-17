extern crate gio;
extern crate glib;
extern crate gtk;
extern crate webkit2gtk;

use gio::BufferedInputStream;
#[cfg(feature = "v2_4")]
use glib::ToVariant;
use gtk::{cairo::RectangleInt, prelude::*, Inhibit, Window, WindowType};
#[cfg(feature = "v2_6")]
use webkit2gtk::UserContentManager;
use webkit2gtk::{
    traits::{SettingsExt, WebContextExt, WebViewExt},
    SnapshotOptions, SnapshotRegion, WebContext, WebView,
};

// * Personal deps
use cairo::{self, ImageSurface};
use std::io::prelude::*;
use std::path::Path;
use std::{fs::File, ops::Deref};

// ! DEBUG UTILS
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    gtk::init().unwrap();
    let window = Window::new(WindowType::Toplevel);
    let context = WebContext::default().unwrap();
    #[cfg(feature = "v2_4")]
    context.set_web_extensions_initialization_user_data(&"webkit".to_variant());
    context.set_web_extensions_directory("../webkit2gtk-webextension-rs/example/target/debug/");
    #[cfg(feature = "v2_6")]
    let webview =
        WebView::new_with_context_and_user_content_manager(&context, &UserContentManager::new());
    #[cfg(not(feature = "v2_6"))]
    let webview = WebView::with_context(&context);
    webview.load_uri("https://crates.io/");
    window.add(&webview);

    let settings = WebViewExt::settings(&webview).unwrap();
    settings.set_enable_developer_extras(true);

    /*let inspector = webview.get_inspector().unwrap();
    inspector.show();*/
    // let mut in_buff = {None}
    window.show_all();

    // webview.run_javascript("alert('Hello');", None::<&gio::Cancellable>, |_result| {});
    // #[cfg(feature = "v2_22")]
    // webview.run_javascript("42", None::<&gio::Cancellable>, |result| match result {
    //     Ok(result) => {
    //         let value = result.js_value();
    //         println!("is_boolean: {}", value.is_boolean(&context));
    //         println!("is_number: {}", value.is_number(&context));
    //         println!("{:?}", value.to_number(&context));
    //         println!("{:?}", value.to_boolean(&context));
    //     }
    //     Err(error) => println!("{}", error),
    // });

    let cancellable = gio::Cancellable::new();
    webview.run_javascript("42", Some(&cancellable), |result| match result {
        Ok(result) => {
            let context = result.global_context().unwrap();
            let value = result.value().unwrap();
            println!("is_boolean: {}", value.is_boolean(&context));
            println!("is_number: {}", value.is_number(&context));
            println!("{:?}", value.to_number(&context));
            println!("{:?}", value.to_boolean(&context));
            // * SCREENSHOT
        }
        Err(error) => println!("{}", error),
    });

    webview.connect_load_changed(move |x, _| {
        println!("SNAP SNAP");
        let wv_cp = x.clone();
        do_screenshot(wv_cp);
    });

    window.connect_delete_event(move |_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}

fn do_screenshot(webv: WebView) {
    println!("STARTING SNAPSHOT...");
    let wv = webv.clone();
    wv.snapshot(
        SnapshotRegion::Visible,
        SnapshotOptions::NONE,
        None::<&gio::Cancellable>,
        move |snap_result| match snap_result {
            Ok(data) => {
                let extents = RectangleInt {
                    x: 0,
                    y: 0,
                    height: 800,
                    width: 600,
                };
                let old_surface = data.clone();
                let mapped_img = data.map_to_image(Some(extents));
                match mapped_img {
                    Ok(a) => {
                        let file = File::create("/tmp/screen.png");
                        match file {
                            Ok(mut f) => {
                                println!("{:?}", a);
                                // let mut vec = Vec::new();
                                let img = ImageSurface::try_from(old_surface);
                                match img {
                                    Ok(img_to_write)=> {img_to_write.write_to_png(&mut f).expect("can't write to file");
                                            println!("Resulting vector: {:?}", f);}
                                    Err(_) => println!("nonono, error on img_write"),
                                }
                            }
                            Err(e) => {
                                print!("{:?}", e)
                            }
                        }
                    }
                    Err(e) => {}
                }
            }
            Err(e) => {
                panic!()
            }
        },
    )
}

use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;

extern crate gtk;
use gtk::prelude::*;

extern crate glib;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;

extern crate madder;
use madder::*;

fn create_ui(timeline: &serializer::TimelineStructure) {
    let timeline: Rc<RefCell<Timeline>> = Rc::new(RefCell::new(Timeline::new_from_structure(timeline)));
    Timeline::setup(timeline.clone());

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_default_size(640,600);
    window.set_title("MADDER");

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let menubar = gtk::MenuBar::new();
    vbox.pack_start(&menubar, true, true, 0);

    {
        let timeline_item = gtk::MenuItem::new_with_label("タイムライン");
        menubar.append(&timeline_item);

        let timeline_menu = gtk::Menu::new();
        timeline_item.set_submenu(&timeline_menu);

        {
            let video_item = gtk::MenuItem::new_with_label("動画");
            timeline_menu.append(&video_item);

            let window = window.clone();
            let timeline = timeline.clone();
            video_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.mkv");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                let timeline: &RefCell<Timeline> = timeline.borrow();
                timeline.borrow_mut().register(&serializer::ComponentStructure {
                    component_type: serializer::ComponentType::Video,
                    start_time: 0,
                    end_time: 100,
                    entity: dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                    coordinate: (0,0),
                });

                dialog.destroy();
            });
        }

        {
            let image_item = gtk::MenuItem::new_with_label("画像");
            timeline_menu.append(&image_item);

            let window = window.clone();
            let timeline = timeline.clone();
            image_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("画像を選択"), Some(&window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.png");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                let timeline: &RefCell<Timeline> = timeline.borrow();
                timeline.borrow_mut().register(&serializer::ComponentStructure {
                    component_type: serializer::ComponentType::Image,
                    start_time: 0,
                    end_time: 100,
                    entity: dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                    coordinate: (0,0),
                });

                dialog.destroy();
            });
        }

        {
            let text_item = gtk::MenuItem::new_with_label("テキスト");
            timeline_menu.append(&text_item);

            let timeline = timeline.clone();
            text_item.connect_activate(move |_| {
                let timeline: &RefCell<Timeline> = timeline.borrow();
                timeline.borrow_mut().register(&serializer::ComponentStructure {
                    component_type: serializer::ComponentType::Text,
                    start_time: 0,
                    end_time: 100,
                    entity: "dummy entity".to_string(),
                    coordinate: (50,50),
                });
            });
        }
    }

    let canvas = gtk::DrawingArea::new();
    canvas.set_size_request(640, 480);

    vbox.pack_start(&canvas, true, true, 0);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let entry = gtk::Entry::new();
    let go_btn = gtk::Button::new();

    hbox.pack_start(&entry, true, true, 0);
    hbox.pack_start(&go_btn, true, true, 5);
    vbox.pack_start(&hbox, true, true, 5);

    let btn = gtk::Button::new();
    btn.set_label("render");

    {
        let timeline = timeline.clone();
        btn.connect_clicked(move |_| {
            let timeline: &RefCell<Timeline> = &timeline.borrow();
            timeline.borrow_mut().write("output/output.avi", 100, 5);
        });
    }

    {
        let timeline: Rc<RefCell<Timeline>> = timeline.clone();
        canvas.connect_draw(move |_,cr| {
            let timeline: &RefCell<Timeline> = timeline.borrow();
            timeline.borrow_mut().renderer(cr)
        });
    }

    {
        let entry = entry.clone();
        let entry = Rc::new(entry);

        let timeline: Rc<RefCell<Timeline>> = timeline.clone();

        go_btn.set_label("Go");
        go_btn.connect_clicked(move |_| {
            if let Ok(time) = entry.get_text().unwrap().parse::<u64>() {
                let timeline: &RefCell<Timeline> = timeline.borrow();
                timeline.borrow_mut().seek_to(time * gst::MSECOND);
            }
        });
    }

    vbox.pack_start(&btn, true, true, 5);

    {
        let timeline: &RefCell<Timeline> = timeline.borrow();
        let timeline: &Timeline = &timeline.borrow();
        vbox.pack_start(&timeline.get_widget(), true, true, 5);
    }

    window.add(&vbox);

    window.show_all();

    window.connect_delete_event(move |_,_| {
        gtk::main_quit();
        Inhibit(false)
    });
}

fn main() {
    gtk::init().expect("Gtk initialization error");
    gst::init().expect("Gstreamer initialization error");

    use std::env;
    let args = env::args().collect::<Vec<String>>();

    let timeline =
        if args.len() >= 2 {
            serializer::TimelineStructure::new_from_file(&args[1])
        } else {
            serializer::TimelineStructure {
                size: (640,480),
                components: Box::new([]),
            }
        };

    create_ui(&timeline);

    gtk::main();
}

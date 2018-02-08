use std::env;
use std::rc::Rc;
use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::cmp;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;

extern crate gtk;
use gtk::prelude::*;

extern crate glib;

extern crate gdk;
use gdk::prelude::*;

extern crate gdk_pixbuf;

extern crate cairo;
extern crate pango;

extern crate madder;
use madder::{Timeline, serializer};

fn create_ui(timeline: &serializer::TimelineStructure) {
    let timeline: Rc<RefCell<Timeline>> = Rc::new(RefCell::new(Timeline::new_from_structure(timeline)));

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_default_size(640,600);

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
//                timeline.add(dialog.get_filename().unwrap());
                dialog.destroy();
            });

            let image_item = gtk::MenuItem::new_with_label("画像");
            timeline_menu.append(&image_item);
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

    let fixed = gtk::Fixed::new();
    fixed.set_size_request(640,100);
    vbox.pack_start(&fixed, true, true, 0);

    let new_component_widget = move |fixed: &gtk::Fixed, label_text: &str, offset_x: i32, width: i32| {
        let evbox = gtk::EventBox::new();
        fixed.put(&evbox, offset_x, 50);

        let label = gtk::Label::new(label_text);
        evbox.add(&label);
        label.override_background_color(gtk::StateFlags::NORMAL, &gdk::RGBA::red());
        label.set_ellipsize(pango::EllipsizeMode::End);
        label.set_size_request(width,30);

        let evbox = evbox.clone();
        let fixed = fixed.clone();
        let offset: Rc<Cell<i32>> = Rc::new(Cell::new(0));

        {
            let offset = offset.clone();
            evbox.connect_button_press_event(move |evbox,button| {
                let (rx,_) = evbox.get_parent().unwrap().get_window().unwrap().get_position();
                let (x,_) = button.get_position();
                let offset: &Cell<i32> = offset.borrow();
                offset.set(rx + x as i32);
                Inhibit(false)
            });
        }

        {
            let fixed = fixed.clone();

            evbox.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
            evbox.connect_motion_notify_event(move |evbox,motion| {
                let (x,_) = motion.get_position();
                let evbox_window = motion.get_window().unwrap();
                let (rx,_) = evbox_window.get_position();

                {
                    let GRAB_EDGE = 5;
                    if (evbox_window.get_width() - x as i32) <= GRAB_EDGE {
                        evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "e-resize"));
                    } else if (x as i32) <= GRAB_EDGE {
                        evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "w-resize"));
                    } else {
                        evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "default"));
                    }
                }

                if motion.get_state().contains(gdk::ModifierType::BUTTON1_MASK) {
                    let offset: &Cell<i32> = offset.borrow();
                    let x_max = evbox.get_parent().unwrap().get_allocation().width - evbox.get_allocation().width;

                    fixed.move_(evbox, cmp::max(cmp::min(rx + x as i32 - offset.get(), x_max), 0), 50);
                }

                Inhibit(false)
            });
        }
    };

    let timeline: &RefCell<Timeline> = timeline.borrow();

    for elem in &timeline.borrow().elements {
        let fixed = fixed.clone();
        let time_to_length = |p: gst::ClockTime| p.mseconds().unwrap() as i32;
        new_component_widget(&fixed, &elem.name, time_to_length(elem.start_time), time_to_length(elem.end_time - elem.start_time));
    }

    window.add(&vbox);

    window.show_all();

    window.connect_delete_event(move |_,_| {
        gtk::main_quit();
        Inhibit(false)
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Give the video filepath as a first argument");
    }

    gtk::init().expect("Gtk initialization error");
    gst::init().expect("Gstreamer initialization error");

    create_ui(&serializer::TimelineStructure {
        size: (640, 480),
        components: Box::new([
            serializer::ComponentStructure {
                component_type: serializer::ComponentType::Video,
                start_time: 120 * gst::MSECOND,
                end_time: 320 * gst::MSECOND,
                entity: args[1].to_string(),
                coordinate: (0,0),
            },
            serializer::ComponentStructure {
                component_type: serializer::ComponentType::Image,
                start_time: 0 * gst::MSECOND,
                end_time: 100 * gst::MSECOND,
                entity: args[2].to_string(),
                coordinate: (100,200),
            }
        ])
    });

    gtk::main();
}

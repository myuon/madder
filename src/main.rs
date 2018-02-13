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

fn create_ui(editor: &serializer::EditorStructure) {
    let editor: Rc<RefCell<Editor>> = Rc::new(RefCell::new(Editor::new_from_structure(editor)));
    Editor::setup(editor.clone());

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_default_size(640,600);
    window.set_title("MADDER");

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let menubar = gtk::MenuBar::new();
    vbox.pack_start(&menubar, true, true, 0);

    {
        let editor_item = gtk::MenuItem::new_with_label("タイムライン");
        menubar.append(&editor_item);

        let editor_menu = gtk::Menu::new();
        editor_item.set_submenu(&editor_menu);

        {
            let video_item = gtk::MenuItem::new_with_label("動画");
            editor_menu.append(&video_item);

            let window = window.clone();
            let editor = editor.clone();
            video_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.mkv");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                let editor: &RefCell<Editor> = editor.borrow();
                editor.borrow_mut().register(&serializer::ComponentStructure {
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
            editor_menu.append(&image_item);

            let window = window.clone();
            let editor = editor.clone();
            image_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("画像を選択"), Some(&window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.png");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                let editor: &RefCell<Editor> = editor.borrow();
                editor.borrow_mut().register(&serializer::ComponentStructure {
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
            editor_menu.append(&text_item);

            let editor = editor.clone();
            text_item.connect_activate(move |_| {
                let editor: &RefCell<Editor> = editor.borrow();
                editor.borrow_mut().register(&serializer::ComponentStructure {
                    component_type: serializer::ComponentType::Text,
                    start_time: 0,
                    end_time: 100,
                    entity: "dummy entity".to_string(),
                    coordinate: (50,50),
                });
            });
        }
    }

    {
        let canvas = &editor.as_ref().borrow().canvas;
        vbox.pack_start(canvas, true, true, 0);
    }

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let entry = gtk::Entry::new();
    let go_btn = gtk::Button::new();

    hbox.pack_start(&entry, true, true, 0);
    hbox.pack_start(&go_btn, true, true, 5);
    vbox.pack_start(&hbox, true, true, 5);

    let btn = gtk::Button::new();
    btn.set_label("render");

    {
        let editor = editor.clone();
        btn.connect_clicked(move |_| {
            let editor: &RefCell<Editor> = &editor.borrow();
            editor.borrow_mut().write("output/output.avi", 100, 5);
        });
    }

    {
        let entry = entry.clone();
        let entry = Rc::new(entry);

        let editor: Rc<RefCell<Editor>> = editor.clone();

        go_btn.set_label("Go");
        go_btn.connect_clicked(move |_| {
            if let Ok(time) = entry.get_text().unwrap().parse::<u64>() {
                let editor: &RefCell<Editor> = editor.borrow();
                editor.borrow_mut().seek_to(time * gst::MSECOND);
            }
        });
    }

    vbox.pack_start(&btn, true, true, 5);

    {
        let editor: &RefCell<Editor> = editor.borrow();
        let editor: &Editor = &editor.borrow();
        editor.set_pack_start(&vbox);
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

    let editor =
        if args.len() >= 2 {
            serializer::EditorStructure::new_from_file(&args[1])
        } else {
            serializer::EditorStructure {
                size: (640,480),
                components: Box::new([]),
            }
        };

    create_ui(&editor);

    gtk::main();
}

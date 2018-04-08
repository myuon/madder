use std::cmp;
use std::rc::Rc;
use std::path::PathBuf;
use std::io::{BufWriter, Write};
use std::fs::File;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;

use gst::prelude::*;

extern crate gtk;
extern crate glib;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate cairo;
extern crate pango;

use gtk::prelude::*;
use gdk::prelude::*;
use gdk_pixbuf::prelude::*;

extern crate serde_json;

extern crate madder_core;
use madder_core::*;

use widget::*;
use gtk_impl;
use ui_impl;

use WINDOW_NUMBER;

pub struct App {
    editor: Editor,
    timeline: TimelineWidget<ui_impl::TimelineComponentRenderer>,
    canvas: gtk::DrawingArea,
    property: PropertyViewerWidget,
    effect_viewer: EffectViewer<ui_impl::EffectComponentRenderer>,
    selected_component_index: Option<usize>,
    window: gtk::Window,
    project_file_path: Option<PathBuf>,
    _menu_for_timeline: Option<Rc<gtk::Menu>>,
}

impl App {
    fn new_with(editor: Editor, width: i32, length: gst::ClockTime) -> App {
        let prop_width = 250;

        App {
            editor: editor,
            timeline: TimelineWidget::new(width, 130, cmp::max(width + prop_width, length.mseconds().unwrap() as i32)),
            canvas: gtk::DrawingArea::new(),
            property: PropertyViewerWidget::new(prop_width),
            effect_viewer: EffectViewer::new(),
            selected_component_index: None,
            window: gtk::Window::new(gtk::WindowType::Toplevel),
            project_file_path: None,
            _menu_for_timeline: None,
        }
    }

    pub fn new(width: i32, height: i32, length: gst::ClockTime) -> App {
        App::new_with(Editor::new(width, height, length), width, length)
    }

    pub fn new_from_json(json: serde_json::Value) -> App {
        let editor = Editor::new_from_json(json);
        let width = editor.get_value(Pointer::from_str("/width")).as_i32().unwrap();
        let length = editor.get_value(Pointer::from_str("/length")).as_time().unwrap();

        App::new_with(editor, width, length)
    }

    pub fn new_from_file(path: &str) -> App {
        let editor = Editor::new_from_file(path);
        let width = editor.get_value(Pointer::from_str("/width")).as_i32().unwrap();
        let length = editor.get_value(Pointer::from_str("/length")).as_time().unwrap();

        App::new_with(editor, width, length)
    }

    pub fn start_instant_preview(&mut self, parent: &gtk::Box) {
        let pipeline = gst::Pipeline::new(None);
        let videoconvert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let appsrc = gst::ElementFactory::make("appsrc", None).unwrap();
        appsrc.set_property("emit-signals", &glib::Value::from(&true)).unwrap();

        let (sink, widget) = if let Some(gtkglsink) = gst::ElementFactory::make("gtkglsink", None) {
            let glsinkbin = gst::ElementFactory::make("glsinkbin", None).unwrap();
            glsinkbin.set_property("sink", &gtkglsink.to_value()).unwrap();

            let widget = gtkglsink.get_property("widget").unwrap();
            (glsinkbin, widget.get::<gtk::Widget>().unwrap())
        } else { panic!(); };

        let width = self.editor.get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32;
        let height = self.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32;
        widget.set_size_request(width, height);
        parent.pack_start(&widget, false, false, 0);
        self.canvas.hide();
        widget.show();

        pipeline.add_many(&[&appsrc, &videoconvert, &sink]).unwrap();
        gst::Element::link_many(&[&appsrc, &videoconvert, &sink]).unwrap();

        let appsrc = appsrc.clone().dynamic_cast::<gsta::AppSrc>().unwrap();
        let info = gstv::VideoInfo::new(
            gstv::VideoFormat::Rgb,
            self.editor.get_value(Pointer::from_str("/width")).as_i64().unwrap() as u32 / 2,
            self.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as u32 / 2,
        ).fps(gst::Fraction::new(20,1)).build().unwrap();
        appsrc.set_caps(&info.to_caps().unwrap());
        appsrc.set_property_format(gst::Format::Time);
        appsrc.set_max_bytes(1);
        appsrc.set_property_block(true);

        let bus = pipeline.get_bus().unwrap();

        {
            let pipeline = pipeline.clone();
            bus.add_watch(move |_,msg| {
                use gst::MessageView;

                match msg.view() {
                    MessageView::Eos(..) => {
                        pipeline.set_state(gst::State::Null).into_result().unwrap();
                        glib::Continue(false)
                    },
                    MessageView::Error(err) => {
                        println!(
                            "Error from {:?}: {:?}",
                            err.get_error(),
                            err.get_debug(),
                        );
                        pipeline.set_state(gst::State::Null).into_result().unwrap();
                        glib::Continue(false)
                    }
                    _ => glib::Continue(true),
                }
            });
        }

        pipeline.set_state(gst::State::Playing).into_result().unwrap();

        let mut pos = 0;
        let self_ = self as *mut Self;
        gtk::idle_add(move || {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let width = self_.editor.get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32;
            let height = self_.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32;
            let mut buffer = gst::Buffer::with_size((width*height*3/4) as usize).unwrap();
            {
                let buffer = buffer.get_mut().unwrap();

                buffer.set_pts(pos * 500 * gst::MSECOND);
                let position = self_.editor.get_value(Pointer::from_str("/position")).as_u64().unwrap();
                self_.editor.seek_to((position + 500) * gst::MSECOND);

                let mut data = buffer.map_writable().unwrap();
                let mut data = data.as_mut_slice();

                let pixbuf = self_.editor.get_current_pixbuf();
                let pixbuf = pixbuf.scale_simple(width / 2, height / 2, gdk_pixbuf::InterpType::Nearest).unwrap();
                let pixels = unsafe { pixbuf.get_pixels() };

                use std::io::Write;
                data.write_all(pixels).unwrap();
            }
            appsrc.push_buffer(buffer).into_result().unwrap();
            pos += 1;

            let position = self_.editor.get_value(Pointer::from_str("/position")).as_time().unwrap();
            let editor = &self_.editor;
            let elems = editor.elements.iter().filter(|&elem| {
                elem.component_type == ComponentType::Sound
                    && elem.as_ref().start_time <= position
                    && position <= elem.start_time + elem.length
            }).collect::<Vec<_>>().clone();

            elems.iter().for_each(|elem| {
                elem.get_audio_pipeline().unwrap().set_state(gst::State::Playing).into_result().unwrap();
            });

            Continue(true)
        });
    }

    fn queue_draw(&self) {
        self.canvas.queue_draw();
        self.timeline.queue_draw();
    }

    fn remove_selected(&mut self) {
        let index = self.selected_component_index.unwrap();
        self.editor.patch_once(Operation::Remove(
            Pointer::from_str(&format!("/components/{}", index))
        ), ContentType::Value).unwrap();
        self.selected_component_index = None;
        self.property.clear();
        self.queue_draw();
    }

    fn save_to_file_with_dialog(&self) {
        let dialog = gtk::FileChooserDialog::new(Some("保存先のファイルを指定"), Some(&self.window), gtk::FileChooserAction::Save);
        dialog.add_button("保存", 0);
        dialog.run();
        let path = dialog.get_filename().unwrap();
        dialog.destroy();

        self.save_to_file(path);
    }

    fn save_to_file(&self, path: PathBuf) {
        let mut buf = BufWriter::new(File::create(path).unwrap());
        buf.write(&format!("{:#}", self.editor.get_value(Pointer::from_str(""))).as_bytes()).unwrap();
    }

    fn open_with_dialog(&mut self) {
        let dialog = gtk::FileChooserDialog::new(Some("開くファイルを指定"), Some(&self.window), gtk::FileChooserAction::Open);
        dialog.add_button("開く", 0);
        dialog.run();
        let path = dialog.get_filename().unwrap();
        dialog.destroy();

        let mut app = App::new_from_file(path.to_str().unwrap());
        app.create_ui();
    }

    fn create_menu(&mut self) -> gtk::MenuBar {
        let menubar = gtk::MenuBar::new();
        let file_item = {
            let file_item = gtk::MenuItem::new_with_label("ファイル");
            let file_menu = gtk::Menu::new();
            file_item.set_submenu(&file_menu);

            let new_project = gtk::MenuItem::new_with_label("新規作成");
            let open = gtk::MenuItem::new_with_label("開く");
            let save_as = gtk::MenuItem::new_with_label("名前を付けて保存");
            let save = gtk::MenuItem::new_with_label("上書き保存");
            let output = gtk::MenuItem::new_with_label("動画の書き出し");

            file_menu.append(&new_project);
            file_menu.append(&open);
            file_menu.append(&save_as);
            file_menu.append(&save);
            file_menu.append(&output);

            new_project.connect_activate(move |_| {
                let editor = json!({
                    "components": [],
                    "width": 640,
                    "height": 480,
                    "length": 900000,
                });
                let mut app = App::new_from_json(editor);
                app.create_ui();
            });

            let self_ = self as *mut Self;
            open.connect_activate(move |_| {
                let self_ = unsafe { self_.as_mut().unwrap() };

                self_.open_with_dialog();
            });

            let self_ = self as *mut Self;
            save_as.connect_activate(move |_| {
                let self_ = unsafe { self_.as_mut().unwrap() };

                self_.save_to_file_with_dialog();
            });

            let self_ = self as *mut Self;
            save.connect_activate(move |_| {
                let self_ = unsafe { self_.as_mut().unwrap() };

                match self_.project_file_path {
                    Some(ref path) => self_.save_to_file(path.to_path_buf()),
                    None => self_.save_to_file_with_dialog(),
                }
            });

            let self_ = self as *mut Self;
            output.connect_activate(move |_| {
                let self_ = unsafe { self_.as_mut().unwrap() };

                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&self_.window), gtk::FileChooserAction::Save);
                dialog.add_button("出力", 0);
                dialog.run();
                let path = dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string();
                dialog.destroy();

                let window = gtk::Window::new(gtk::WindowType::Popup);
                let progress_bar = gtk::ProgressBar::new();
                let label = gtk::Label::new("進捗…");
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
                window.add(&vbox);
                window.set_transient_for(&self_.window);
                window.set_position(gtk::WindowPosition::CenterOnParent);

                vbox.pack_start(&label, true, true, 0);
                vbox.pack_start(&progress_bar, true, true, 0);
                progress_bar.set_pulse_step(0.0);

                window.show_all();

                let progress_bar = progress_bar.clone();
                self_.editor.write_init(&path, 100, 5);

                idle_add(move || {
                    let (cont, frac) = self_.editor.write_next();
                    progress_bar.set_fraction(frac);

                    if cont == false {
                        window.destroy();
                    }
                    Continue(cont)
                });
            });

            file_item
        };
        let editor_item = {
            let editor_item = gtk::MenuItem::new_with_label("タイムライン");
            editor_item.set_submenu(&self.timeline.menu);
            editor_item
        };
        let project_item = {
            let project_item = gtk::MenuItem::new_with_label("プロジェクト");
            let project_menu = gtk::Menu::new();
            project_item.set_submenu(&project_menu);

            let project_info = gtk::MenuItem::new_with_label("情報");
            project_menu.append(&project_info);

            let self_ = self as *mut Self;
            project_info.connect_activate(move |_| {
                let self_ = unsafe { self_.as_mut().unwrap() };

                let dialog = gtk::MessageDialog::new(
                    Some(&self_.window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Info,
                    gtk::ButtonsType::Ok,
                    &serde_json::to_string(&json!({
                        "size": (self_.editor.get_value(Pointer::from_str("/width")),
                                 self_.editor.get_value(Pointer::from_str("/height"))),
                        "components": self_.editor.get_value(Pointer::from_str("/components")).as_array().unwrap().len(),
                    })).unwrap(),
                );

                dialog.run();
                dialog.destroy();
            });

            project_item
        };

        menubar.append(&file_item);
        menubar.append(&editor_item);
        menubar.append(&project_item);

        menubar
    }

    pub fn create_ui(&mut self) {
        let self_ = self as *mut Self;
        self.timeline.connect_get_component = Box::new(move |index| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            serde_json::from_value::<Component>(self_.editor.get_value(Pointer::from_str(&format!("/components/{}", index)))).unwrap()
        });

        let self_ = self as *mut Self;
        self.timeline.connect_new_component = Box::new(move |value| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.editor.patch_once(Operation::Add(
                Pointer::from_str("/components"),
                value,
            ), ContentType::Value).unwrap();
        });
        self.timeline.connect_get_component = Box::new(move |index| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            serde_json::from_value::<Component>(self_.editor.get_value(Pointer::from_str(&format!("/components/{}", index)))).unwrap()
        });

        let self_ = self as *mut Self;
        self.timeline.connect_get_objects(Box::new(move || {
            let self_ = unsafe { self_.as_mut().unwrap() };

            serde_json::from_value::<Vec<Component>>(self_.editor.get_value(Pointer::from_str("/components"))).unwrap().iter().enumerate().map(|(i,component)| {
                let entity = serde_json::from_value::<Attribute>(self_.editor.get_attr(Pointer::from_str(&format!("/components/{}/prop/entity", i)))).unwrap();

                let obj = BoxObject::new(
                    component.start_time.mseconds().unwrap() as i32,
                    component.length.mseconds().unwrap() as i32,
                    i
                ).label(format!("{:?}", entity))
                    .selected(Some(i) == self_.selected_component_index)
                    .layer_index(component.layer_index);

                ui_impl::TimelineComponentRenderer {
                    object: obj,
                    object_type: component.component_type.clone(),
                }
            }).collect()
        }));

        let self_ = self as *mut Self;
        self.timeline.connect_render_object(Box::new(move |robj, scaler, cr| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            robj.hscaled(scaler).renderer(cr, &|p| self_.editor.elements[robj.object.index].peek(p));
        }));

        let self_ = self as *mut Self;
        self.timeline.connect_set_component_attr = Box::new(move |index, attr, value| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.editor.patch_once(Operation::Add(
                Pointer::from_str(&format!("/components/{}/{}", index, attr)),
                json!(value),
            ), ContentType::Attribute).unwrap();
        });

        let self_ = self as *mut Self;
        self.timeline.connect_select_component = Box::new(move |index: usize| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.property.clear();

            let self__ = self_ as *mut Self;
            self_.property.append_page("component", GridPage::new(
                self_.property.width,
                serde_json::from_value(self_.editor.get_attr(Pointer::from_str(&format!("/components/{}", index)))).unwrap(),
                &|prop_name, prop: Attribute, _| {
                    let self_ = unsafe { self__.as_mut().unwrap() };

                    let prop_name = Rc::new(prop_name.to_string());

                    let self__ = self_ as *mut Self;
                    gtk_impl::edit_type_as_widget(&prop, vec![], Rc::new(move |new_prop, tracker| {
                        let self_ = unsafe { self__.as_mut().unwrap() };
                        let prop = serde_json::from_value::<Attribute>(self_.editor.get_attr(Pointer::from_str(&format!("/components/{}/{}", index, *prop_name)))).unwrap().clone();
                        if let Some(new_prop) = new_prop {
                            self_.editor.patch_once(Operation::Add(
                                Pointer::from_str(&format!("/components/{}/{}", index, prop_name.as_str())),
                                json!(gtk_impl::recover_property(prop, tracker, new_prop)),
                            ), ContentType::Attribute).unwrap();
                        }

                        self_.queue_draw();
                    }))
                },
            ));

            let self__ = self_ as *mut Self;
            self_.property.append_page("property", GridPage::new(
                self_.property.width,
                serde_json::from_value(self_.editor.get_attr(Pointer::from_str(&format!("/components/{}/prop", index)))).unwrap(),
                &|prop_name, prop, prop_index| {
                    let self_ = unsafe { self__.as_mut().unwrap() };

                    let prop_index = Rc::new(prop_index);
                    let prop_name = Rc::new(prop_name.to_string());
                    let self__ = self_ as *mut Self;
                    gtk_impl::edit_type_as_widget(&prop, vec![], Rc::new(move |new_prop, tracker| {
                        let self_ = unsafe { self__.as_mut().unwrap() };
                        let prop = serde_json::from_value::<Vec<(String, Attribute)>>(self_.editor.get_attr(Pointer::from_str(&format!("/components/{}/prop", index)))).unwrap()[*prop_index].1.clone();
                        if let Some(new_prop) = new_prop {
                            self_.editor.patch_once(Operation::Add(
                                Pointer::from_str(&format!("/components/{}/prop/{}", index, prop_name.as_str())),
                                json!(gtk_impl::recover_property(prop, tracker, new_prop)),
                            ), ContentType::Attribute).unwrap();
                        }

                        self_.queue_draw();
                    }))
                },
            ));

            let self__ = self_ as *mut Self;
            self_.property.append_page("effect", BoxPage::new(
                self_.property.width,
                serde_json::from_value::<Vec<Vec<(String, Attribute)>>>(self_.editor.get_attr(Pointer::from_str(&format!("/components/{}/effect", index)))).unwrap(),
                &|prop_vec: Vec<(String, Attribute)>, prop_index: usize| {
                    let self_ = unsafe { self__.as_mut().unwrap() };

                    let prop_index = Rc::new(prop_index);

                    let expander = gtk::Expander::new("Effect");
                    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    expander.add(&vbox);
                    vbox.set_margin_start(10);

                    for (i, (prop_name, prop)) in prop_vec.into_iter().enumerate() {
                        let prop_index = prop_index.clone();

                        let self__ = self_ as *mut Self;
                        vbox.pack_start(&gtk_impl::edit_type_as_widget(&prop, vec![], Rc::new(move |new_prop,tracker| {
                            let self_ = unsafe { self__.as_mut().unwrap() };
                            let prop = serde_json::from_value::<Vec<Vec<(String, Attribute)>>>(self_.editor.get_attr(Pointer::from_str(&format!("/components/{}/effect", index)))).unwrap()[*prop_index][i].1.clone();
                            if let Some(new_prop) = new_prop {
                                self_.editor.patch_once(Operation::Add(
                                    Pointer::from_str(&format!("/components/{}/effect/{}/{}", index, *prop_index, prop_name.as_str())),
                                    json!(gtk_impl::recover_property(prop, tracker, new_prop)),
                                ), ContentType::Attribute).unwrap();
                            }

                            self_.queue_draw();
                        })), true, true, 0);
                    }

                    expander.dynamic_cast().unwrap()
                },
                Box::new(move || {
                    let self_ = unsafe { self__.as_mut().unwrap() };

                    self_.editor.patch_once(Operation::Add(
                        Pointer::from_str(&format!("/components/{}/effect", index)),
                        json!({
                            "effect_type": "CoordinateX",
                            "transition": "Linear",
                            "start_value": 0.0,
                            "end_value": 0.0,
                        }),
                    ), ContentType::Value).unwrap();
                    (self_.timeline.connect_select_component)(index);
                }),
                Box::new(move |i| {
                    let self_ = unsafe { self__.as_mut().unwrap() };

                    self_.editor.patch_once(Operation::Remove(
                        Pointer::from_str(&format!("/components/{}/effect/{}", index, i)),
                    ), ContentType::Value).unwrap();
                    (self_.timeline.connect_select_component)(index);
                }),
            ));

            self_.property.append_page("info", BoxPage::new(
                self_.property.width,
                vec![self_.editor.get_value(Pointer::from_str(&format!("/components/{}/info", index)))],
                &|t,_| {
                    gtk::Label::new(t.as_str()).dynamic_cast().unwrap()
                },
                Box::new(|| {}),
                Box::new(|_| {}),
            ));

            self_.selected_component_index = Some(index);
            self_.queue_draw();
        });

        let self_ = self as *mut Self;
        self.timeline.connect_select_component_menu = Box::new(move |index, position| {
            let self_ = unsafe { self_.as_mut().unwrap() };
            let menu = gtk::Menu::new();
            let split_component_here = {
                let split_component_here = gtk::MenuItem::new_with_label("オブジェクトをこの位置で分割");

                let self__ = self_ as *mut Self;
                split_component_here.connect_activate(move |_| {
                    let self_ = unsafe { self__.as_mut().unwrap() };
                    let this_component = serde_json::from_value::<Component>(self_.editor.get_value(Pointer::from_str(&format!("/components/{}", index)))).unwrap();
                    let mut this = self_.editor.get_value(Pointer::from_str(&format!("/components/{}", index)));
                    this.as_object_mut().unwrap()["start_time"] = json!(position.mseconds().unwrap());
                    this.as_object_mut().unwrap()["length"] = json!(this_component.length.mseconds().unwrap() - position.mseconds().unwrap());

                    self_.editor.patch(vec![
                        Operation::Add(
                            Pointer::from_str(&format!("/components/{}/length", index)),
                            json!((position - this_component.start_time).mseconds().unwrap()),
                        ),
                        Operation::Add(
                            Pointer::from_str("/components"),
                            this,
                        ),
                    ], ContentType::Value).unwrap();

                    self_.queue_draw();
                });

                split_component_here
            };
            let open_effect_window = {
                let open_effect_window = gtk::MenuItem::new_with_label("エフェクトウィンドウを開く");

                let self__ = self_ as *mut Self;
                open_effect_window.connect_activate(move |_| {
                    let self_ = unsafe { self__.as_mut().unwrap() };
                    self_.effect_viewer.setup();
                    self_.effect_viewer.popup();
                });

                open_effect_window
            };

            menu.append(&split_component_here);
            menu.append(&open_effect_window);
            menu
        });

        let self_ = self as *mut Self;
        self.effect_viewer.connect_get_effect = Box::new(move |index| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            serde_json::from_value(self_.editor.get_value(Pointer::from_str(&format!("/components/{}/effect/{}", self_.selected_component_index.expect("App::selected_component_index is None"), index)))).unwrap()
        });

        let self_ = self as *mut Self;
        self.effect_viewer.connect_get_objects(Box::new(move || {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.editor
                .get_value(Pointer::from_str(&format!("/components/{}/effect", self_.selected_component_index.unwrap())))
                .as_array().unwrap()
                .iter()
                .map(|obj| serde_json::from_value::<Effect>(obj.clone()).unwrap())
                .enumerate()
                .map(|(i,obj)| { ui_impl::EffectComponentRenderer::new(i,obj) })
                .collect()
        }));

        self.effect_viewer.connect_render_object(Box::new(move |robj: ui_impl::EffectComponentRenderer, scaler, cr| {
            robj.renderer(scaler, cr)
        }));

        let self_ = self as *mut Self;
        self.effect_viewer.connect_new_point = Box::new(move |eff_index, point| {
            let self_ = unsafe { self_.as_mut().unwrap() };
            let index = self_.selected_component_index.unwrap();
            let current = self_.editor.get_value(Pointer::from_str(&format!("/components/{}/effect/{}/intermeds/value/{}", index, eff_index, point))).as_f64().unwrap();
            self_.editor.patch_once(
                Operation::Add(
                    Pointer::from_str(&format!("/components/{}/effect/{}/intermeds", index, eff_index)),
                    json!(EffectPoint {
                        transition: Transition::Ease,
                        position: point,
                        value: current,
                    }),
                ), ContentType::Value
            ).unwrap();

            self_.effect_viewer.queue_draw();
        });

        self.timeline.create_ui();
        self.timeline.connect_drag_component();

        let self_ = self as *mut Self;
        self.timeline.connect_ruler_seek_time(move |time| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.editor.seek_to(time);
            self_.queue_draw();

            Inhibit(false)
        });

        let self_ = self as *mut Self;
        self.canvas.connect_draw(move |_,cr| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            cr.set_source_pixbuf(&self_.editor.get_current_pixbuf(), 0f64, 0f64);
            cr.paint();
            Inhibit(false)
        });

        self.canvas.set_size_request(
            self.editor.get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32,
            self.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32
        );
        self.window.set_size_request(
            self.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32,
            self.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32 + 200
        );
        self.window.set_title("madder");

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&self.create_menu(), true, true, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&self.canvas, true, true, 0);
        hbox.pack_end(self.property.as_widget(), true, true, 0);

        let self_ = self as *mut Self;
        self.property.connect_remove(Box::new(move || {
            let self_ = unsafe { self_.as_mut().unwrap() };
            self_.remove_selected();
        }));

        vbox.pack_start(&hbox, true, true, 0);
        vbox.pack_start(self.timeline.as_widget(), true, true, 5);

        let self_ = self as *mut Self;
        let hbox_ = hbox.clone();
        let btn = gtk::Button::new();
        btn.set_label("start preview");
        btn.connect_clicked(move |_| {
            let self_ = unsafe { self_.as_mut().unwrap() };
            self_.start_instant_preview(&hbox_);
        });
        vbox.pack_start(&btn, false, false, 0);

        self.window.add(&vbox);
        self.window.show_all();
        self.window.connect_delete_event(move |window,_| {
            window.destroy();

            unsafe {
                if WINDOW_NUMBER == 1 {
                    gtk::main_quit();
                } else {
                    WINDOW_NUMBER -= 1;
                }
            }

            Inhibit(false)
        });

        unsafe {
            WINDOW_NUMBER += 1;
        }
    }
}

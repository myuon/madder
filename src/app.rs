use std::cmp;
use std::rc::Rc;
use std::cell::RefCell;
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

extern crate serde_json;

extern crate madder_core;
use madder_core::*;
use util::gtk_util;

use widget::*;
use gtk_impl;
use ui_impl;

use WINDOW_NUMBER;

pub struct App {
    editor: Editor,
    timeline: Rc<RefCell<TimelineWidget<App>>>,
    canvas: gtk::DrawingArea,
    property: PropertyViewerWidget,
    effect_viewer: Rc<RefCell<EffectViewer<App>>>,
    selected_component_index: Option<usize>,
    window: gtk::Window,
    project_file_path: Option<PathBuf>,
}

impl TimelineWidgetI for App {
    type Renderer = ui_impl::TimelineComponentRenderer;

    fn get_component(&self, index: usize) -> component::Component {
        serde_json::from_value::<Component>(self.editor.get_value(Pointer::from_str(&format!("/components/{}", index)))).unwrap()
    }

    fn get_objects(&self) -> Vec<Self::Renderer> {
        serde_json::from_value::<Vec<Component>>(self.editor.get_value(Pointer::from_str("/components"))).unwrap().iter().enumerate().map(|(i,component)| {
            let entity = serde_json::from_value::<Attribute>(self.editor.get_attr(Pointer::from_str(&format!("/components/{}/prop/entity", i)))).unwrap();

            let obj = BoxObject::new(
                component.start_time.mseconds().unwrap() as i32,
                component.length.mseconds().unwrap() as i32,
                i
            ).label(format!("{:?}", entity))
                .selected(Some(i) == self.selected_component_index)
                .layer_index(component.layer_index);

            ui_impl::TimelineComponentRenderer {
                object: obj,
                object_type: component.component_type.clone(),
            }
        }).collect()
    }

    fn do_render(&self, robj: Self::Renderer, scaler: f64, cr: &cairo::Context) {
        robj.hscaled(scaler).renderer(cr, &|p| self.editor.elements[robj.object.index].peek(p));
    }

    fn set_component_attr(&mut self, index: usize, attr: &str, value: Attribute) {
        self.editor.patch_once(Operation::Add(
            Pointer::from_str(&format!("/components/{}/{}", index, attr)),
            json!(value),
        ), ContentType::Value).unwrap();
    }

    fn connect_select_component(self_: Rc<RefCell<Self>>, index: usize) {
        self_.borrow().property.clear();

        let self__ = self_.clone();
        self_.borrow().property.append_page("component", GridPage::new(
            self_.borrow().property.width,
            serde_json::from_value(self__.borrow().editor.get_attr(Pointer::from_str(&format!("/components/{}", index)))).unwrap(),
            &|prop_name, prop: Attribute, index| {
                let prop_name = Rc::new(prop_name.to_string());
                let self__ = self__.clone();

                gtk_impl::edit_type_as_widget(&prop, vec![], Rc::new(move |new_prop, tracker| {
                    // request the property again, since in this callback the value of property might have been changed
                    let prop = serde_json::from_value::<Attribute>(self__.borrow().editor.get_attr(Pointer::from_str(&format!("/components/{}/{}", index, *prop_name)))).unwrap().clone();
                    if let Some(new_prop) = new_prop {
                        self__.borrow_mut().editor.patch_once(Operation::Add(
                            Pointer::from_str(&format!("/components/{}/{}", index, prop_name.as_str())),
                            json!(gtk_impl::recover_property(prop, tracker, new_prop)),
                        ), ContentType::Attribute).unwrap();
                    }

                    self__.borrow().queue_draw();
                }))
            },
        ));

        let self__ = self_.clone();
        self_.borrow().property.append_page("property", GridPage::new(
            self_.borrow().property.width,
            serde_json::from_value(self_.borrow().editor.get_attr(Pointer::from_str(&format!("/components/{}/prop", index)))).unwrap(),
            &|prop_name, prop, prop_index| {
                let prop_index = Rc::new(prop_index);
                let prop_name = Rc::new(prop_name.to_string());
                let self__ = self__.clone();

                gtk_impl::edit_type_as_widget(&prop, vec![], Rc::new(move |new_prop, tracker| {
                    // request the property again, since in this callback the value of property might have been changed
                    let prop = serde_json::from_value::<Vec<(String, Attribute)>>(self__.borrow().editor.get_attr(Pointer::from_str(&format!("/components/{}/prop", index)))).unwrap()[*prop_index].1.clone();
                    if let Some(new_prop) = new_prop {
                        self__.borrow_mut().editor.patch_once(Operation::Add(
                            Pointer::from_str(&format!("/components/{}/prop/{}", index, prop_name.as_str())),
                            json!(gtk_impl::recover_property(prop, tracker, new_prop)),
                        ), ContentType::Attribute).unwrap();
                    }

                    self__.borrow().queue_draw();
                }))
            },
        ));

        let self__ = self_.clone();
        let self___ = self_.clone();
        let self____ = self_.clone();
        self_.borrow().property.append_page("effect", BoxPage::new(
            self_.borrow().property.width,
            serde_json::from_value::<Vec<Vec<(String, Attribute)>>>(self_.borrow().editor.get_attr(Pointer::from_str(&format!("/components/{}/effect", index)))).unwrap(),
            &|prop_vec: Vec<(String, Attribute)>, prop_index: usize| {
                let prop_index = Rc::new(prop_index);
                let self__ = self__.clone();

                let expander = gtk::Expander::new("Effect");
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
                expander.add(&vbox);
                vbox.set_margin_left(10);

                for (i, (prop_name, prop)) in prop_vec.into_iter().enumerate() {
                    let prop_index = prop_index.clone();
                    let self__ = self__.clone();

                    vbox.pack_start(&gtk_impl::edit_type_as_widget(&prop, vec![], Rc::new(move |new_prop,tracker| {
                        // request the property again, since in this callback the value of property might have been changed
                        let prop = serde_json::from_value::<Vec<Vec<(String, Attribute)>>>(self__.borrow().editor.get_attr(Pointer::from_str(&format!("/components/{}/effect", index)))).unwrap()[*prop_index][i].1.clone();
                        if let Some(new_prop) = new_prop {
                            self__.borrow_mut().editor.patch_once(Operation::Add(
                                Pointer::from_str(&format!("/components/{}/effect/{}/{}", index, *prop_index, prop_name.as_str())),
                                json!(gtk_impl::recover_property(prop, tracker, new_prop)),
                            ), ContentType::Attribute).unwrap();
                        }

                        self__.borrow().queue_draw();
                    })), true, true, 0);
                }

                expander.dynamic_cast().unwrap()
            },
            Box::new(move || {
                self___.borrow_mut().editor.patch_once(Operation::Add(
                    Pointer::from_str(&format!("/components/{}/effect", index)),
                    json!({
                        "effect_type": "CoordinateX",
                        "transition": "Linear",
                        "start_value": 0.0,
                        "end_value": 0.0,
                    }),
                ), ContentType::Value).unwrap();
                TimelineWidgetI::connect_select_component(self___.clone(), index);
            }),
            Box::new(move |i| {
                self____.borrow_mut().editor.patch_once(Operation::Remove(
                    Pointer::from_str(&format!("/components/{}/effect/{}", index, i)),
                ), ContentType::Value).unwrap();
                TimelineWidgetI::connect_select_component(self____.clone(), index);
            }),
        ));

        self_.borrow().property.append_page("info", BoxPage::new(
            self_.borrow().property.width,
            vec![self_.borrow().editor.get_value(Pointer::from_str(&format!("/components/{}/info", index)))],
            &|t,_| {
                gtk::Label::new(t.as_str()).dynamic_cast().unwrap()
            },
            Box::new(|| {}),
            Box::new(|_| {}),
        ));

        self_.borrow_mut().selected_component_index = Some(index);
        self_.borrow().queue_draw();
    }

    fn connect_select_component_menu(self_: Rc<RefCell<Self>>, index: usize, position: gst::ClockTime) -> gtk::Menu {
        let menu = gtk::Menu::new();
        let split_component_here = {
            let split_component_here = gtk::MenuItem::new_with_label("オブジェクトをこの位置で分割");

            let self_ = self_.clone();
            split_component_here.connect_activate(move |_| {
                let this_component = serde_json::from_value::<Component>(self_.borrow().editor.get_value(Pointer::from_str(&format!("/components/{}", index)))).unwrap();
                let mut this = self_.borrow().editor.get_value(Pointer::from_str(&format!("/components/{}", index)));
                this.as_object_mut().unwrap()["start_time"] = json!(position.mseconds().unwrap());
                this.as_object_mut().unwrap()["length"] = json!(this_component.length.mseconds().unwrap() - position.mseconds().unwrap());

                self_.borrow_mut().editor.patch(vec![
                    Operation::Add(
                        Pointer::from_str(&format!("/components/{}/length", index)),
                        json!((position - this_component.start_time).mseconds().unwrap()),
                    ),
                    Operation::Add(
                        Pointer::from_str("/components"),
                        this,
                    ),
                ], ContentType::Value).unwrap();

                self_.borrow().queue_draw();
            });

            split_component_here
        };
        let open_effect_window = {
            let open_effect_window = gtk::MenuItem::new_with_label("エフェクトウィンドウを開く");

            let self_ = self_.clone();
            open_effect_window.connect_activate(move |_| {
                let effect_viewer = self_.borrow().effect_viewer.clone();

                let self____ = self_.clone();
                self____.borrow_mut().selected_component_index = Some(index);

                EffectViewer::setup(effect_viewer.clone());

                effect_viewer.borrow().popup();
            });

            open_effect_window
        };

        menu.append(&split_component_here);
        menu.append(&open_effect_window);
        menu
    }
}

impl EffectViewerI for App {
    type Renderer = ui_impl::EffectComponentRenderer;

    fn get_effect(&self, effect_index: usize) -> component::Effect {
        serde_json::from_value(self.editor.get_value(Pointer::from_str(&format!("/components/{}/effect/{}", self.selected_component_index.expect("App::selected_component_index is None"), effect_index)))).unwrap()
    }

    fn get_effects(&self) -> Vec<Self::Renderer> {
        self.editor
            .get_value(Pointer::from_str(&format!("/components/{}/effect", self.selected_component_index.unwrap())))
            .as_array().unwrap()
            .iter()
            .map(|obj| serde_json::from_value::<Effect>(obj.clone()).unwrap())
            .enumerate()
            .map(|(i,obj)| { ui_impl::EffectComponentRenderer::new(i,obj) })
            .collect()
    }

    fn do_render(&self, renderer: Self::Renderer, scaler: f64, cr: &cairo::Context) {
        renderer.renderer(scaler, cr)
    }

    fn connect_new_point(&mut self, eff_index: usize, point: f64) {
        let index = self.selected_component_index.unwrap();
        let current = self.editor.get_value(Pointer::from_str(&format!("/components/{}/effect/{}/intermeds/value/{}", index, eff_index, point))).as_f64().unwrap();
        self.editor.patch_once(
            Operation::Add(
                Pointer::from_str(&format!("/components/{}/effect/{}/intermeds", index, eff_index)),
                json!(EffectPoint {
                    transition: Transition::Ease,
                    position: point,
                    value: current,
                }),
            ), ContentType::Value
        ).unwrap();

        self.effect_viewer.borrow().queue_draw();
    }
}

impl App {
    fn new_with(editor: Editor, width: i32, length: gst::ClockTime) -> Rc<RefCell<App>> {
        let prop_width = 250;

        Rc::new(RefCell::new(App {
            editor: editor,
            timeline: TimelineWidget::new(width, 130, cmp::max(width + prop_width, length.mseconds().unwrap() as i32)),
            canvas: gtk::DrawingArea::new(),
            property: PropertyViewerWidget::new(prop_width),
            effect_viewer: EffectViewer::new(),
            selected_component_index: None,
            window: gtk::Window::new(gtk::WindowType::Toplevel),
            project_file_path: None,
        }))
    }

    pub fn new(width: i32, height: i32, length: gst::ClockTime) -> Rc<RefCell<App>> {
        App::new_with(Editor::new(width, height, length), width, length)
    }

    pub fn new_from_json(json: serde_json::Value) -> Rc<RefCell<App>> {
        let editor = Editor::new_from_json(json);
        let width = editor.get_value(Pointer::from_str("/width")).as_i32().unwrap();
        let length = editor.get_value(Pointer::from_str("/length")).as_time().unwrap();

        App::new_with(editor, width, length)
    }

    pub fn new_from_file(path: &str) -> Rc<RefCell<App>> {
        let editor = Editor::new_from_file(path);
        let width = editor.get_value(Pointer::from_str("/width")).as_i32().unwrap();
        let length = editor.get_value(Pointer::from_str("/length")).as_time().unwrap();

        App::new_with(editor, width, length)
    }

    pub fn start_instant_preview(self_: Rc<RefCell<App>>, parent: &gtk::Box) {
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

        let width = self_.borrow().editor.get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32;
        let height = self_.borrow().editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32;
        widget.set_size_request(width, height);
        parent.pack_start(&widget, false, false, 0);
        self_.borrow().canvas.hide();
        widget.show();

        pipeline.add_many(&[&appsrc, &videoconvert, &sink]).unwrap();
        gst::Element::link_many(&[&appsrc, &videoconvert, &sink]).unwrap();

        let appsrc = appsrc.clone().dynamic_cast::<gsta::AppSrc>().unwrap();
        let info = gstv::VideoInfo::new(
            gstv::VideoFormat::Rgb,
            self_.borrow().editor.get_value(Pointer::from_str("/width")).as_i64().unwrap() as u32 / 2,
            self_.borrow().editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as u32 / 2,
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
        let self__ = self_.clone();
        gtk::idle_add(move || {
            let width = self_.borrow().editor.get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32;
            let height = self_.borrow().editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32;
            let mut buffer = gst::Buffer::with_size((width*height*3/4) as usize).unwrap();
            {
                let buffer = buffer.get_mut().unwrap();

                buffer.set_pts(pos * 500 * gst::MSECOND);
                let position = self__.borrow().editor.get_value(Pointer::from_str("/position")).as_u64().unwrap();
                self__.borrow_mut().editor.seek_to((position + 500) * gst::MSECOND);

                let mut data = buffer.map_writable().unwrap();
                let mut data = data.as_mut_slice();

                let pixbuf = self__.borrow().editor.get_current_pixbuf();
                let pixbuf = pixbuf.scale_simple(width / 2, height / 2, gtk_util::GdkInterpType::Nearest.to_i32()).unwrap();
                let pixels = unsafe { pixbuf.get_pixels() };

                use std::io::Write;
                data.write_all(pixels).unwrap();
            }
            appsrc.push_buffer(buffer).into_result().unwrap();
            pos += 1;

            let position = self_.borrow().editor.get_value(Pointer::from_str("/position")).as_time().unwrap();
            let editor = &self__.borrow().editor;
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
        self.timeline.borrow().queue_draw();
    }

    fn remove_selected(self_: Rc<RefCell<App>>) {
        let index = self_.borrow().selected_component_index.unwrap();
        self_.borrow_mut().editor.patch_once(Operation::Remove(
            Pointer::from_str(&format!("/components/{}", index))
        ), ContentType::Value).unwrap();
        self_.borrow_mut().selected_component_index = None;
        self_.borrow().property.clear();
        self_.borrow().queue_draw();
    }

    fn save_to_file_with_dialog(self_: Rc<RefCell<App>>) {
        let dialog = gtk::FileChooserDialog::new(Some("保存先のファイルを指定"), Some(&self_.borrow().window), gtk::FileChooserAction::Save);
        dialog.add_button("保存", 0);
        dialog.run();
        let path = dialog.get_filename().unwrap();
        dialog.destroy();

        App::save_to_file(self_.clone(), path);
    }

    fn save_to_file(self_: Rc<RefCell<App>>, path: PathBuf) {
        let mut buf = BufWriter::new(File::create(path).unwrap());
        buf.write(&format!("{:#}", self_.borrow().editor.get_value(Pointer::from_str(""))).as_bytes()).unwrap();
    }

    fn open_with_dialog(self_: Rc<RefCell<App>>) {
        let dialog = gtk::FileChooserDialog::new(Some("開くファイルを指定"), Some(&self_.borrow().window), gtk::FileChooserAction::Open);
        dialog.add_button("開く", 0);
        dialog.run();
        let path = dialog.get_filename().unwrap();
        dialog.destroy();

        let app = App::new_from_file(path.to_str().unwrap());
        App::create_ui(app);
    }

    fn create_menu(self_: Rc<RefCell<App>>) -> gtk::MenuBar {
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
                let app = App::new_from_json(editor);
                App::create_ui(app);
            });

            let self__ = self_.clone();
            open.connect_activate(move |_| {
                App::open_with_dialog(self__.clone());
            });

            let self__ = self_.clone();
            save_as.connect_activate(move |_| {
                App::save_to_file_with_dialog(self__.clone());
            });

            let self__ = self_.clone();
            save.connect_activate(move |_| {
                let self___ = self__.clone();

                match self__.borrow().project_file_path {
                    Some(ref path) => App::save_to_file(self___, path.to_path_buf()),
                    None => App::save_to_file_with_dialog(self___),
                }
            });

            let self__ = self_.clone();
            output.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&self__.borrow().window), gtk::FileChooserAction::Save);
                dialog.add_button("出力", 0);
                dialog.run();
                let path = dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string();
                dialog.destroy();

                let self__ = self__.clone();
                let window = gtk::Window::new(gtk::WindowType::Popup);
                let progress_bar = gtk::ProgressBar::new();
                let label = gtk::Label::new("進捗…");
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
                window.add(&vbox);
                window.set_transient_for(&self__.borrow().window);
                window.set_position(gtk::WindowPosition::CenterOnParent);

                vbox.pack_start(&label, true, true, 0);
                vbox.pack_start(&progress_bar, true, true, 0);
                progress_bar.set_pulse_step(0.0);

                window.show_all();

                let progress_bar = progress_bar.clone();
                self__.borrow_mut().editor.write_init(&path, 100, 5);

                idle_add(move || {
                    let (cont, frac) = self__.borrow_mut().editor.write_next();
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
            let editor_menu = gtk::Menu::new();
            editor_item.set_submenu(&editor_menu);

            let app = self_.borrow();
            app.timeline.borrow_mut().create_menu(&editor_menu);

            let video_item = gtk::MenuItem::new_with_label("動画");
            let image_item = gtk::MenuItem::new_with_label("画像");
            let text_item = gtk::MenuItem::new_with_label("テキスト");
            editor_menu.append(&video_item);
            editor_menu.append(&image_item);
            editor_menu.append(&text_item);

            let self__ = self_.clone();
            video_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&self__.borrow().window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.mkv");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                self__.borrow_mut().editor.patch_once(Operation::Add(
                    Pointer::from_str("/components"),
                    json!({
                        "component_type": "Video",
                        "start_time": 0,
                        "length": 100,
                        "layer_index": 0,
                        "prop": {
                            "entity": dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                        }
                    }),
                ), ContentType::Value).unwrap();

                self__.borrow().queue_draw();
                dialog.destroy();
            });

            let self__ = self_.clone();
            image_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("画像を選択"), Some(&self__.borrow().window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.png");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                self__.borrow_mut().editor.patch_once(Operation::Add(
                    Pointer::from_str("/components"),
                    json!({
                        "component_type": "Image",
                        "start_time": 0,
                        "length": 100,
                        "layer_index": 0,
                        "prop": {
                            "entity": dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                        }
                    }),
                ), ContentType::Value).unwrap();

                self__.borrow().queue_draw();
                dialog.destroy();
            });

            let self__ = self_.clone();
            text_item.connect_activate(move |_| {
                self__.borrow_mut().editor.patch_once(Operation::Add(
                    Pointer::from_str("/components"),
                    json!({
                        "component_type": "Text",
                        "start_time": 0,
                        "length": 100,
                        "layer_index": 0,
                        "prop": {
                            "entity": "dummy entity",
                            "coordinate": [50, 50],
                        }
                    }),
                ), ContentType::Value).unwrap();

                self__.borrow().queue_draw();
            });

            editor_item
        };
        let project_item = {
            let project_item = gtk::MenuItem::new_with_label("プロジェクト");
            let project_menu = gtk::Menu::new();
            project_item.set_submenu(&project_menu);

            let project_info = gtk::MenuItem::new_with_label("情報");
            project_menu.append(&project_info);

            let self__ = self_.clone();
            project_info.connect_activate(move |_| {
                let dialog = gtk::MessageDialog::new(
                    Some(&self__.borrow().window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Info,
                    gtk::ButtonsType::Ok,
                    &serde_json::to_string(&json!({
                        "size": (self__.borrow().editor.get_value(Pointer::from_str("/width")),
                                 self__.borrow().editor.get_value(Pointer::from_str("/height"))),
                        "components": self__.borrow().editor.get_value(Pointer::from_str("/components")).as_array().unwrap().len(),
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

    pub fn create_ui(self_: Rc<RefCell<App>>) {
        let app = self_.borrow();

        let self__ = self_.clone();
        app.effect_viewer.borrow_mut().set_model(self__);

        let self__ = self_.clone();
        app.timeline.borrow_mut().set_model(self__);

        TimelineWidget::create_ui(app.timeline.clone());

        TimelineWidget::connect_drag_component(app.timeline.clone());

        let self__ = self_.clone();
        TimelineWidget::connect_ruler_seek_time(app.timeline.clone(), move |time| {
            self__.borrow_mut().editor.seek_to(time);
            self__.borrow().queue_draw();

            Inhibit(false)
        });

        let self__ = self_.clone();
        app.canvas.connect_draw(move |_,cr| {
            cr.set_source_pixbuf(&self__.borrow().editor.get_current_pixbuf(), 0f64, 0f64);
            cr.paint();
            Inhibit(false)
        });

        app.canvas.set_size_request(
            app.editor.get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32,
            app.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32
        );
        app.window.set_size_request(
            app.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32,
            app.editor.get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32 + 200
        );
        app.window.set_title("madder");

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&App::create_menu(self_.clone()), true, true, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&app.canvas, true, true, 0);
        hbox.pack_end(app.property.as_widget(), true, true, 0);

        let self__ = self_.clone();
        app.property.connect_remove(Box::new(move || {
            App::remove_selected(self__.clone());
        }));

        vbox.pack_start(&hbox, true, true, 0);
        vbox.pack_start(app.timeline.borrow().as_widget(), true, true, 5);

        let self__ = self_.clone();
        let hbox_ = hbox.clone();
        let btn = gtk::Button::new();
        btn.set_label("start preview");
        btn.connect_clicked(move |_| {
            App::start_instant_preview(self__.clone(), &hbox_);
        });
        vbox.pack_start(&btn, false, false, 0);

        app.window.add(&vbox);
        app.window.show_all();
        app.window.connect_delete_event(move |window,_| {
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

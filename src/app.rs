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
use gdk_pixbuf::prelude::*;

extern crate relm;
use relm::*;

extern crate serde_json;

extern crate madder_core;
use madder_core::*;

use widget::*;
use gtk_impl;
use ui_impl;

use WINDOW_NUMBER;

pub struct Model {
    editor: Rc<RefCell<Editor>>,
    selected_component_index: Rc<RefCell<Option<usize>>>,
    project_file_path: Option<PathBuf>,
    relm: Relm<App>,
}

#[derive(Msg)]
pub enum AppMsg {
    Quit(Rc<gtk::Window>),
    RemoveSelected,
    SeekTime(gst::ClockTime),
    SetComponentAttr(usize, &'static str, Attribute),
    NewComponent(serde_json::Value),
    SelectComponent(usize),
    SetAttr(WidgetType, Pointer),
    NewIntermedPoint(usize, f64),

    NewProject,
    OpenProject,
    SaveProject,
    SaveAsProject,
    ProjectInfo,
    OutputMovie,
}

pub struct App {
    model: Model,
    window: gtk::Window,
    canvas: gtk::DrawingArea,
    timeline: relm::Component<TimelineWidget<ui_impl::TimelineComponentRenderer>>,
    prop_viewer: relm::Component<PropertyViewerWidget>,
    effect_viewer: relm::Component<EffectViewerWidget<ui_impl::EffectComponentRenderer>>,
}

impl Update for App {
    type Model = Model;
    type ModelParam = serde_json::Value;
    type Msg = AppMsg;

    fn model(relm: &Relm<Self>, value: serde_json::Value) -> Model {
        Model {
            editor: Rc::new(RefCell::new(Editor::new_from_json(value))),
            selected_component_index: Rc::new(RefCell::new(None)),
            project_file_path: None,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: AppMsg) {
        use self::AppMsg::*;

        match event {
            Quit(window) => {
                let window = &window;
                window.destroy();

                unsafe {
                    if WINDOW_NUMBER == 1 {
                        gtk::main_quit();
                    } else {
                        WINDOW_NUMBER -= 1;
                    }
                }
            },
            RemoveSelected => {
//                self.remove_selected();
            },
            SeekTime(time) => {
                self.model.editor.borrow_mut().seek_to(time);
                self.canvas.queue_draw();
                self.timeline.stream().emit(TimelineMsg::RulerQueueDraw);
            },
            SetComponentAttr(index, name, attr) => {
                self.model.editor.borrow_mut().patch_once(Operation::Add(
                    Pointer::from_str(&format!("/components/{}/{}", index, name)),
                    json!(attr),
                ), ContentType::Attribute).unwrap();
            },
            NewComponent(value) => {
                self.model.editor.borrow_mut().patch_once(Operation::Add(
                    Pointer::from_str("/components"),
                    value,
                ), ContentType::Value).unwrap();
            },
            SelectComponent(index) => {
                *self.model.selected_component_index.borrow_mut() = Some(index);

                let editor = self.model.editor.borrow();
                self.prop_viewer.stream().emit(PropertyMsg::ClearPage(0));
                self.prop_viewer.stream().emit(PropertyMsg::ClearPage(1));

                self.prop_viewer.stream().emit(PropertyMsg::AppendGridWidget(
                    0,
                    serde_json::from_value::<Vec<_>>(editor.get_attr(Pointer::from_str(&format!(
                        "/components/{}/common",
                        self.model.selected_component_index.borrow().unwrap()))
                    )).unwrap().into_iter().map(|(key,value): (String, Attribute)| {
                        (key.to_string(), gtk_impl::attribute_to_widget_type(value.clone()), format!("/components/{}/{}", self.model.selected_component_index.borrow().unwrap(), key))
                    }).collect(),
                ));
                self.prop_viewer.stream().emit(PropertyMsg::AppendGridWidget(
                    0,
                    serde_json::from_value::<Vec<_>>(editor.get_attr(Pointer::from_str(&format!(
                        "/components/{}/prop",
                        self.model.selected_component_index.borrow().unwrap()))
                    )).unwrap().into_iter().map(|(key,value): (String, Attribute)| {
                        (key.to_string(), gtk_impl::attribute_to_widget_type(value.clone()), format!("/components/{}/prop/{}", self.model.selected_component_index.borrow().unwrap(), key))
                    }).collect(),
                ));

                self.prop_viewer.stream().emit(PropertyMsg::AppendVBoxWidget(
                    1,
                    serde_json::from_value::<Vec<Vec<_>>>(editor.get_attr(Pointer::from_str(&format!(
                        "/components/{}/effect",
                        self.model.selected_component_index.borrow().unwrap()))
                    )).unwrap().into_iter().enumerate().map(|(i,attrs): (_,Vec<(String,Attribute)>)| {
                        (WidgetType::Expander(
                            "effect".to_string(),
                            Box::new(WidgetType::Grid(attrs.into_iter().map(|(key,value)| {
                                (key, gtk_impl::attribute_to_widget_type(value.clone()))
                            }).collect()))
                        ), format!("/components/{}/effect/{}", self.model.selected_component_index.borrow().unwrap(), i))
                    }).collect(),
                ));

                self.timeline.stream().emit(TimelineMsg::QueueDraw);
                self.effect_viewer.stream().emit(EffectMsg::QueueDraw);
            },
            SetAttr(widget_type, pointer) => {
                self.model.editor.borrow_mut().patch_once(Operation::Add(
                    pointer,
                    gtk_impl::widget_type_to_value(widget_type),
                ), ContentType::Value).unwrap();

                self.timeline.stream().emit(TimelineMsg::QueueDraw);
                self.canvas.queue_draw();
            },
            NewIntermedPoint(eff_index, point) => {
                let index = self.model.selected_component_index.borrow().unwrap();
                let current = self.model.editor.borrow().get_value(Pointer::from_str(&format!("/components/{}/effect/{}/intermeds/value/{}", index, eff_index, point))).as_f64().unwrap();

                self.model.editor.borrow_mut().patch_once(
                    Operation::Add(
                        Pointer::from_str(&format!("/components/{}/effect/{}/intermeds", index, eff_index)),
                        json!(EffectPoint {
                            transition: Transition::Ease,
                            position: point,
                            value: current,
                        }),
                    ), ContentType::Value
                ).unwrap();

                self.effect_viewer.widget().queue_draw();
            },
            NewProject => {
                let json = json!({
                    "components": [],
                    "width": 640,
                    "height": 480,
                    "length": 900000,
                });
                App::run(json).unwrap();
            },
            OpenProject => {
                let dialog = gtk::FileChooserDialog::new(Some("開くファイルを指定"), Some(&self.window), gtk::FileChooserAction::Open);
                dialog.add_button("開く", 0);
                dialog.run();
                let path = dialog.get_filename().unwrap();
                dialog.destroy();

                let file = ::std::fs::File::open(&path).unwrap();
                *self.model.editor.borrow_mut() = Editor::new_from_json(serde_json::from_reader(file).unwrap());

                self.timeline.stream().emit(TimelineMsg::QueueDraw);
            },
            SaveProject => {
                match self.model.project_file_path {
                    Some(ref path) => {
                        let path = path.to_path_buf();
                        let mut buf = BufWriter::new(File::create(path).unwrap());
                        buf.write(&format!("{:#}", self.model.editor.borrow().get_value(Pointer::from_str(""))).as_bytes()).unwrap();
                    },
                    None => self.model.relm.stream().emit(AppMsg::SaveAsProject),
                }
            },
            SaveAsProject => {
                let dialog = gtk::FileChooserDialog::new(Some("保存先のファイルを指定"), Some(&self.window), gtk::FileChooserAction::Save);
                dialog.add_button("保存", 0);
                dialog.run();
                let path = dialog.get_filename().unwrap();
                dialog.destroy();

                self.model.project_file_path = Some(path);
                self.model.relm.stream().emit(AppMsg::SaveProject);
            },
            ProjectInfo => {
                let editor = self.model.editor.borrow();

                let dialog = gtk::MessageDialog::new(
                    Some(&self.window),
                    gtk::DialogFlags::MODAL,
                    gtk::MessageType::Info,
                    gtk::ButtonsType::Ok,
                    &serde_json::to_string(&json!({
                        "size": (editor.get_value(Pointer::from_str("/width")),
                                 editor.get_value(Pointer::from_str("/height"))),
                        "components": editor.get_value(Pointer::from_str("/components")).as_array().unwrap().len(),
                    })).unwrap(),
                );

                dialog.run();
                dialog.destroy();
            },
            OutputMovie => {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&self.window), gtk::FileChooserAction::Save);
                dialog.add_button("出力", 0);
                dialog.run();
                let path = dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string();
                dialog.destroy();

                let window = gtk::Window::new(gtk::WindowType::Popup);
                let progress_bar = gtk::ProgressBar::new();
                let label = gtk::Label::new("進捗…");
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
                window.add(&vbox);
                window.set_transient_for(&self.window);
                window.set_position(gtk::WindowPosition::CenterOnParent);

                vbox.pack_start(&label, true, true, 0);
                vbox.pack_start(&progress_bar, true, true, 0);
                progress_bar.set_pulse_step(0.0);

                window.show_all();

                let progress_bar = progress_bar.clone();
                self.model.editor.borrow_mut().write_init(&path, 100, 5);

                let editor = self.model.editor.clone();
                idle_add(move || {
                    let (cont, frac) = editor.borrow_mut().write_next();
                    progress_bar.set_fraction(frac);

                    if cont == false {
                        window.destroy();
                    }
                    Continue(cont)
                });
            },
        }
    }
}

impl Widget for App {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn init_view(&mut self) {
        unsafe {
            WINDOW_NUMBER += 1;
        }

        self.canvas.set_size_request(
            self.model.editor.borrow().get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32,
            self.model.editor.borrow().get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32
        );

        let editor = self.model.editor.clone();
        self.canvas.connect_draw(move |_,cr| {
            cr.set_source_pixbuf(&editor.borrow().get_current_pixbuf(), 0f64, 0f64);
            cr.paint();
            Inhibit(false)
        });

        let editor = self.model.editor.clone();
        self.timeline.stream().emit(TimelineMsg::ConnectGetComponent(Box::new(move |index| {
            serde_json::from_value::<component::Component>(editor.borrow().get_value(Pointer::from_str(&format!("/components/{}", index)))).unwrap()
        })));
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("madder");
        connect!(relm, window, connect_delete_event(window,_), return (AppMsg::Quit(Rc::new(window.clone())), Inhibit(false)));

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let menu = App::create_menu(relm);
        vbox.pack_start(&menu, true, true, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        vbox.pack_start(&hbox, true, true, 0);

        let canvas = gtk::DrawingArea::new();
        hbox.pack_start(&canvas, true, true, 0);

        let vbox_prop = gtk::Box::new(gtk::Orientation::Vertical, 0);
        hbox.pack_start(&vbox_prop, true, true, 0);

        let switcher = gtk::StackSwitcher::new();
        vbox_prop.pack_start(&switcher, false, false, 0);

        let stack = gtk::Stack::new();
        switcher.set_stack(&stack);
        vbox_prop.pack_start(&stack, true, true, 0);

        let prop_viewer = stack.add_widget::<PropertyViewerWidget>(500);
        stack.set_child_title(prop_viewer.widget(), "Grid");
        prop_viewer.stream().emit(PropertyMsg::AppendPage("property"));
        prop_viewer.stream().emit(PropertyMsg::AppendPage("effect"));

        {
            use self::PropertyMsg::*;
            connect!(prop_viewer@OnChangeAttr(ref widget_type, ref path), relm, AppMsg::SetAttr(widget_type.clone(), path.clone()));
        }

        // remove => (AppMsg::RemoveSelected, ()),

        let effect_viewer = stack.add_widget::<EffectViewerWidget<ui_impl::EffectComponentRenderer>>((
            {
                let editor = model.editor.clone();
                let selected_component_index = model.selected_component_index.clone();
                Rc::new(Box::new(move || {
                    editor.borrow()
                        .get_value(Pointer::from_str(&format!("/components/{}/effect", selected_component_index.borrow().unwrap())))
                        .as_array().unwrap()
                        .iter()
                        .map(|obj| serde_json::from_value::<Effect>(obj.clone()).unwrap())
                        .enumerate()
                        .map(|(i,obj)| { ui_impl::EffectComponentRenderer::new(i,obj) })
                        .collect()
                }))
            },
            {
                Rc::new(Box::new(move |robj, scaler, cr| {
                    robj.renderer(scaler, &cr);
                }))
            },
        ));
        stack.set_child_title(effect_viewer.widget(), "Effect");
        {
            use self::EffectMsg::*;
            connect!(effect_viewer@OnNewIntermedPoint(index, ratio), relm, AppMsg::NewIntermedPoint(index, ratio));
        }

        let timeline = vbox.add_widget::<TimelineWidget<ui_impl::TimelineComponentRenderer>>((
            model.editor.borrow().width,
            130,
            cmp::max(model.editor.borrow().width + 250, model.editor.borrow().length.mseconds().unwrap() as i32),
            {
                let editor = model.editor.clone();
                let selected_component_index = model.selected_component_index.clone();
                Rc::new(Box::new(move || {
                    let components = editor.borrow().get_value(Pointer::from_str("/components"));
                    let selected_component_index = *selected_component_index.borrow();
                    let editor = editor.clone();

                    serde_json::from_value::<Vec<component::Component>>(components).unwrap().iter().enumerate().map(move |(i,component)| {
                        let entity = serde_json::from_value::<Attribute>(editor.borrow().get_attr(Pointer::from_str(&format!("/components/{}/prop/entity", i)))).unwrap();

                        let obj = BoxObject::new(
                            component.start_time.mseconds().unwrap() as i32,
                            component.length.mseconds().unwrap() as i32,
                            i
                        ).label(format!("{:?}", entity))
                            .selected(Some(i) == selected_component_index)
                            .layer_index(component.layer_index);

                        ui_impl::TimelineComponentRenderer {
                            object: obj,
                            object_type: component.component_type.clone(),
                        }
                    }).collect()
                }))
            },
            {
                let editor = model.editor.clone();
                Rc::new(Box::new(move |robj, scaler, cr| {
                    robj.hscaled(scaler).renderer(&cr, &|p| editor.borrow().elements[robj.object.index].peek(p));
                }))
            }
        ));

        {
            use self::TimelineMsg::*;
            connect!(timeline@RulerSeekTime(time), relm, AppMsg::SeekTime(time as u64 * gst::MSECOND));
            connect!(timeline@OnSetComponentAttr(index, name, ref attr), relm, AppMsg::SetComponentAttr(index, name, attr.clone()));
            connect!(timeline@OnNewComponent(ref value), relm, AppMsg::NewComponent(value.clone()));
            connect!(timeline@OnSelectComponent(index), relm, AppMsg::SelectComponent(index));
        }

        let button = gtk::Button::new();
        button.set_label("start preview");
        connect!(relm, button, connect_clicked(_), {
            println!("start instant preview");
        });

        window.add(&vbox);
        window.show_all();

        App {
            model: model,
            window: window,
            canvas: canvas,
            timeline: timeline,
            prop_viewer: prop_viewer,
            effect_viewer: effect_viewer,
        }
    }
}

impl App {
    fn create_menu(relm: &Relm<Self>) -> gtk::MenuBar {
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

            connect!(relm, new_project, connect_activate(_), AppMsg::NewProject);
            connect!(relm, open, connect_activate(_), AppMsg::OpenProject);
            connect!(relm, save, connect_activate(_), AppMsg::SaveProject);
            connect!(relm, save_as, connect_activate(_), AppMsg::SaveAsProject);
            connect!(relm, output, connect_activate(_), AppMsg::OutputMovie);

            file_item
        };

        let editor_item = {
            let editor_item = gtk::MenuItem::new_with_label("タイムライン");
//            editor_item.set_submenu(&self.timeline.menu);

            editor_item
        };

        let project_item = {
            let project_item = gtk::MenuItem::new_with_label("プロジェクト");
            let project_menu = gtk::Menu::new();
            project_item.set_submenu(&project_menu);

            let project_info = gtk::MenuItem::new_with_label("情報");
            project_menu.append(&project_info);

            connect!(relm, project_info, connect_activate(_), AppMsg::ProjectInfo);

            project_item
        };

        menubar.append(&file_item);
        menubar.append(&editor_item);
        menubar.append(&project_item);

        menubar
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

        let width = self.model.editor.borrow().get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32;
        let height = self.model.editor.borrow().get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32;
        widget.set_size_request(width, height);
        parent.pack_start(&widget, false, false, 0);
        self.canvas.hide();
        widget.show();

        pipeline.add_many(&[&appsrc, &videoconvert, &sink]).unwrap();
        gst::Element::link_many(&[&appsrc, &videoconvert, &sink]).unwrap();

        let appsrc = appsrc.clone().dynamic_cast::<gsta::AppSrc>().unwrap();
        let info = gstv::VideoInfo::new(
            gstv::VideoFormat::Rgb,
            self.model.editor.borrow().get_value(Pointer::from_str("/width")).as_i64().unwrap() as u32 / 2,
            self.model.editor.borrow().get_value(Pointer::from_str("/height")).as_i64().unwrap() as u32 / 2,
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
        let editor = self.model.editor.clone();
        gtk::idle_add(move || {
            let width = editor.borrow().get_value(Pointer::from_str("/width")).as_i64().unwrap() as i32;
            let height = editor.borrow().get_value(Pointer::from_str("/height")).as_i64().unwrap() as i32;
            let mut buffer = gst::Buffer::with_size((width*height*3/4) as usize).unwrap();
            {
                let buffer = buffer.get_mut().unwrap();

                buffer.set_pts(pos * 500 * gst::MSECOND);
                let position = editor.borrow().get_value(Pointer::from_str("/position")).as_u64().unwrap();
                editor.borrow_mut().seek_to((position + 500) * gst::MSECOND);

                let mut data = buffer.map_writable().unwrap();
                let mut data = data.as_mut_slice();

                let pixbuf = editor.borrow().get_current_pixbuf();
                let pixbuf = pixbuf.scale_simple(width / 2, height / 2, gdk_pixbuf::InterpType::Nearest).unwrap();
                let pixels = unsafe { pixbuf.get_pixels() };

                use std::io::Write;
                data.write_all(pixels).unwrap();
            }
            appsrc.push_buffer(buffer).into_result().unwrap();
            pos += 1;

            let editor_ptr = &editor.borrow();
            let position = editor_ptr.get_value(Pointer::from_str("/position")).as_time().unwrap();
            let elems = editor_ptr.elements.iter().filter(|&elem| {
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

    /*
    fn remove_selected(&mut self) {
        let index = self.selected_component_index.unwrap();
        self.editor.patch_once(Operation::Remove(
            Pointer::from_str(&format!("/components/{}", index))
        ), ContentType::Value).unwrap();
        self.selected_component_index = None;
        self.property.clear();
        self.queue_draw();
    }
    */
}


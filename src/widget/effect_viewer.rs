use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate cairo;
extern crate gstreamer as gst;
use gtk::prelude::*;
use gdk::prelude::*;

extern crate relm;
use relm::*;

extern crate madder_core;
use madder_core::*;
use widget::*;

pub struct Model<Renderer: AsRef<BoxObject> + 'static> {
    tracking_position: Rc<RefCell<f64>>,
    name_list: gtk::Box,
    connect_get_effect: Box<Fn(usize) -> component::Effect>,
    connect_new_point: Box<Fn(usize, f64)>,
    on_get_object: Rc<Box<Fn() -> Vec<Renderer>>>,
    on_render: Rc<Box<Fn(&Renderer, f64, &cairo::Context)>>,
    relm: Relm<EffectViewerWidget<Renderer>>,
}

#[derive(Msg)]
pub enum EffectMsg {
    QueueDraw,
    Select(BoxObject, gdk::EventButton),
    OnNewIntermedPoint(usize, f64),
}

pub struct EffectViewerWidget<Renderer: AsRef<BoxObject> + 'static> {
    model: Model<Renderer>,
    scrolled: gtk::ScrolledWindow,
    box_viewer: relm::Component<BoxViewerWidget<Renderer>>,
    vbox: gtk::Box,
    graph: gtk::DrawingArea,
}

impl<Renderer> Update for EffectViewerWidget<Renderer> where Renderer: AsRef<BoxObject> + 'static {
    type Model = Model<Renderer>;
    type ModelParam = (Rc<Box<Fn() -> Vec<Renderer>>>, Rc<Box<Fn(&Renderer, f64, &cairo::Context)>>);
    type Msg = EffectMsg;

    fn model(relm: &Relm<Self>, (on_get_object, on_render): Self::ModelParam) -> Model<Renderer> {
        Model {
            tracking_position: Rc::new(RefCell::new(0.0)),
            name_list: gtk::Box::new(gtk::Orientation::Vertical, 0),
            connect_get_effect: Box::new(|_| unreachable!()),
            connect_new_point: Box::new(|_,_| unreachable!()),
            on_get_object: on_get_object,
            on_render: on_render,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: EffectMsg) {
        use self::EffectMsg::*;

        match event {
            QueueDraw => {
                self.graph.queue_draw();
            },
            Select(object, event) => {
                *self.model.tracking_position.borrow_mut() = event.get_position().0;
                self.box_viewer.widget().queue_draw();

                if event.get_button() == 3 {
                    let ratio = event.get_position().0 / object.size().0 as f64;
                    self.model.relm.stream().emit(EffectMsg::OnNewIntermedPoint(object.index, ratio));
                }
            },
            _ => (),
        }
    }
}

impl<Renderer> Widget for EffectViewerWidget<Renderer> where Renderer: AsRef<BoxObject> + 'static {
    type Root = gtk::ScrolledWindow;

    fn root(&self) -> Self::Root {
        self.scrolled.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let scrolled = gtk::ScrolledWindow::new(None, None);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        scrolled.add(&vbox);

        let label = gtk::Label::new("▶ Effect Timeline");
        label.set_halign(gtk::Align::Start);
        vbox.pack_start(&label, false, false, 5);

        let overlay = gtk::Overlay::new();
        vbox.pack_start(&overlay, false, false, 0);

        let tracker = gtk::DrawingArea::new();
        tracker.set_size_request(-1, -1);
        tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });
        overlay.add_overlay(&tracker);
        overlay.set_overlay_pass_through(&tracker, true);

        let tracking_position = model.tracking_position.clone();
        tracker.connect_draw(move |tracker,cr| {
            cr.set_source_rgb(200.0, 0.0, 0.0);

            cr.move_to(*tracking_position.borrow(), 0.0);
            cr.rel_line_to(0.0, tracker.get_allocation().height as f64);
            cr.stroke();

            Inhibit(false)
        });

        let box_viewer = overlay.add_widget::<BoxViewerWidget<Renderer>>((
            200,
            Rc::new(gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 1.0, 10.0, 0.1)),
            model.on_get_object.clone(),
            model.on_render.clone(),
        ));
        {
            use self::BoxViewerMsg::*;
            connect!(box_viewer@OnSelect(ref object, ref event), relm, EffectMsg::Select(object.clone(), event.clone()));
        }

        let label = gtk::Label::new("▶ Effect");
        label.set_halign(gtk::Align::Start);
        vbox.pack_start(&label, false, false, 5);

        let combo = gtk::ComboBoxText::new();
        vbox.pack_start(&combo, false, false, 0);

        for item in &["Transition 1", "Transition 2"] {
            combo.append_text(item);
        }
        combo.set_active(0);

        let graph = gtk::DrawingArea::new();
        graph.set_size_request(-1, 300);
        graph.connect_draw(move |_,cr| {
            cr.move_to(0.0, 150.0);
            cr.set_line_width(1.0);
            cr.set_source_rgb(0.8, 0.8, 0.8);
            cr.rel_line_to(500.0, 0.0);
            cr.stroke();

            cr.move_to(0.0, 150.0);
            cr.set_line_width(2.0);
            cr.set_source_rgb(0.4, 0.4, 0.4);
            cr.curve_to(50.0, 150.0, 400.0, 50.0, 500.0, 0.0);
            cr.stroke();

            Inhibit(false)
        });
        vbox.pack_start(&graph, false, false, 0);

        EffectViewerWidget {
            model: model,
            box_viewer: box_viewer,
            vbox: vbox,
            graph: graph,
            scrolled: scrolled,
        }
    }
}


use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate cairo;
extern crate gstreamer as gst;
use gtk::prelude::*;

extern crate relm;
use relm::*;

extern crate madder_core;
use madder_core::*;
use widget::*;

pub trait HasEffect {
    fn as_effect(&self) -> &Effect;
}

pub struct Model<Renderer: AsRef<BoxObject> + Clone + HasEffect + 'static> {
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
    Select(usize, gdk::EventButton),
    OnNewIntermedPoint(usize, f64),
    OnNewEffect(EffectType),
}

pub struct EffectViewerWidget<Renderer: AsRef<BoxObject> + Clone + HasEffect + 'static> {
    model: Model<Renderer>,
    scrolled: gtk::ScrolledWindow,
    timeline: relm::Component<TimelineWidget<Renderer>>,
    vbox: gtk::Box,
    graph: relm::Component<BezierGraphWidget>,
    combo: gtk::ComboBoxText,
}

impl<Renderer> Update for EffectViewerWidget<Renderer> where Renderer: AsRef<BoxObject> + Clone + HasEffect + 'static {
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

    fn update(&mut self, event: Self::Msg) {
        use self::EffectMsg::*;

        match event {
            QueueDraw => {
                self.graph.widget().queue_draw();
                self.timeline.stream().emit(TimelineMsg::QueueDraw);
            },
            Select(index, event) => {
                let renderer = &(self.model.on_get_object)()[index];
                let (_current, _start, _end, transition) = renderer.as_effect().find_interval(event.get_position().0 / renderer.as_ref().width as f64);
                self.combo.set_active(Transition::transitions().into_iter().position(|x| x == transition).unwrap() as i32);

                let object = renderer.as_ref();
                *self.model.tracking_position.borrow_mut() = event.get_position().0;
                self.timeline.widget().queue_draw();

                if event.get_button() == 3 {
                    let ratio = event.get_position().0 / object.size().0 as f64;
                    self.model.relm.stream().emit(EffectMsg::OnNewIntermedPoint(object.index, ratio));
                }
            },
            _ => (),
        }
    }
}

impl<Renderer> Widget for EffectViewerWidget<Renderer> where Renderer: AsRef<BoxObject> + Clone + HasEffect + 'static {
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

        let timeline = vbox.add_widget::<TimelineWidget<Renderer>>((
            -1,
            200,
            2000,
            model.on_get_object.clone(),
            model.on_render.clone(),
            {
                let menu = gtk::Menu::new();

                for effect in EffectType::types() {
                    let item = gtk::MenuItem::new_with_label(&format!("エフェクト:{:?}を追加", effect));
                    menu.append(&item);
                    connect!(relm, item, connect_activate(_), return (EffectMsg::OnNewEffect(effect.clone()), ()));
                }

                menu
            },
        ));

        {
            use self::TimelineMsg::*;
            connect!(timeline@OnSelectBox(index, ref event), relm, EffectMsg::Select(index, event.clone()));
        }

        let label = gtk::Label::new("▶ Effect");
        label.set_halign(gtk::Align::Start);
        vbox.pack_start(&label, false, false, 5);

        let combo = gtk::ComboBoxText::new();
        vbox.pack_start(&combo, false, false, 0);

        for item in Transition::transitions() {
            combo.append_text(&format!("{:?}", item));
        }

        let graph = vbox.add_widget::<BezierGraphWidget>(());

        EffectViewerWidget {
            model: model,
            vbox: vbox,
            timeline: timeline,
            graph: graph,
            scrolled: scrolled,
            combo: combo,
        }
    }
}


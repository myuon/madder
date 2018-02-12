use std::cmp;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate gtk;
use gtk::prelude::*;

extern crate glib;

extern crate gdk;
use gdk::prelude::*;

extern crate gdk_pixbuf;

extern crate cairo;
extern crate pango;
extern crate pangocairo;

mod avi_renderer;
use avi_renderer::AviRenderer;

#[macro_use] extern crate serde_derive;

pub mod serializer;
use serializer::*;

trait Peekable {
    fn get_duration(&self) -> gst::ClockTime;
    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf>;
}

impl Peekable for gst::Element {
    fn get_duration(&self) -> gst::ClockTime {
        100 * 1000 * gst::MSECOND
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.seek_simple(gst::SeekFlags::FLUSH, time).ok().and_then(|_| {
            self.get_property("last-pixbuf").ok().and_then(|x| x.get::<gdk_pixbuf::Pixbuf>())
        })
    }
}

pub struct Component {
    pub name: String,
    pub start_time: gst::ClockTime,
    pub end_time: gst::ClockTime,
    component: Box<Peekable>,
    coordinate: (i32,i32),
}

impl Component {
    pub fn new_from_structure(structure: &ComponentStructure) -> Component {
        match structure.component_type {
            ComponentType::Video => {
                let mut c = VideoFileComponent::new(structure.entity.as_str(), structure.start_time * gst::MSECOND, structure.coordinate).get_component();
                c.end_time = structure.end_time * gst::MSECOND;
                c
            },
            ComponentType::Image => {
                let mut c = ImageComponent::new(structure.entity.as_str(), structure.start_time * gst::MSECOND, structure.coordinate).get_component();
                c.end_time = structure.end_time * gst::MSECOND;
                c
            },
            ComponentType::Text => {
                let mut c = TextComponent::new(structure.entity.as_str(), (640,480), structure.start_time * gst::MSECOND, structure.coordinate).get_component();
                c.end_time = structure.end_time * gst::MSECOND;
                c
            }
            _ => unimplemented!(),
        }
    }
}

struct RulerWidget(gtk::DrawingArea);

impl RulerWidget {
    fn new(width: i32) -> RulerWidget {
        let ruler = gtk::DrawingArea::new();
        ruler.set_size_request(width,100);

        ruler.connect_draw(move |_, cr| {
            cr.set_line_width(1.0);
            cr.set_source_rgb(0f64, 0f64, 0f64);

            let starting_height = 50f64;

            cr.move_to(0f64, starting_height);
            cr.line_to(width as f64, 50f64);

            let interval_large = 100;
            let interval_large_height = 50;

            let interval = 10;
            let interval_height = 30;

            for x in (0..(width / interval)).map(|x| x * interval) {
                cr.move_to(x as f64, starting_height);

                let h = if x % interval_large == 0 { interval_large_height } else { interval_height };
                cr.rel_line_to(0f64, -h as f64);
            }

            cr.stroke();
            Inhibit(false)
        });

        RulerWidget(ruler)
    }

    fn get_widget(&self) -> &gtk::DrawingArea {
        &self.0
    }
}

pub struct TimelineBuilder {
    fixed: gtk::Fixed,
    ruler: RulerWidget,
    offset: i32,
}

// workaround for sharing a variable within callbacks
impl TimelineBuilder {
    fn new(width: i32) -> Rc<RefCell<TimelineBuilder>> {
        let fixed = gtk::Fixed::new();
        fixed.set_size_request(width, 100);

        Rc::new(RefCell::new(TimelineBuilder {
            fixed: fixed,
            ruler: RulerWidget::new(width),
            offset: 0
        }))
    }

    fn get_widget(&self) -> gtk::Box {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(self.ruler.get_widget(), true, true, 0);
        vbox.pack_start(&self.fixed, true, true, 0);
        vbox
    }

    fn add_component_widget(self_: Rc<RefCell<TimelineBuilder>>, label_text: &str, offset_x: i32, width: i32) {
        let evbox = gtk::EventBox::new();
        evbox.show();

        {
            let builder: &RefCell<TimelineBuilder> = self_.borrow();
            let builder: &TimelineBuilder = &builder.borrow();
            builder.fixed.put(&evbox, offset_x, 50);
        }

        let label = gtk::Label::new(label_text);
        evbox.add(&label);
        label.override_background_color(gtk::StateFlags::NORMAL, &gdk::RGBA::red());
        label.set_ellipsize(pango::EllipsizeMode::End);
        label.set_size_request(width,30);
        label.show();

        {
            let self_: Rc<RefCell<TimelineBuilder>> = self_.clone();
            evbox.connect_button_press_event(move |evbox,button| {
                let (rx,_) = evbox.get_parent().unwrap().get_window().unwrap().get_position();
                let (x,_) = button.get_position();

                let builder: &RefCell<TimelineBuilder> = self_.borrow();
                builder.borrow_mut().offset = rx + x as i32;
                Inhibit(false)
            });
        }

        {
            let self_: Rc<RefCell<TimelineBuilder>> = self_.clone();
            evbox.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
            evbox.connect_motion_notify_event(move |evbox,motion| {
                let (x,_) = motion.get_position();
                let evbox_window = motion.get_window().unwrap();
                let (rx,_) = evbox_window.get_position();

                let grab_edge = 5;
                if (evbox_window.get_width() - x as i32) <= grab_edge {
                    evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "e-resize"));
                } else if (x as i32) <= grab_edge {
                    evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "w-resize"));
                } else {
                    evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "default"));
                }

                if motion.get_state().contains(gdk::ModifierType::BUTTON1_MASK) {
                    let x_max = evbox.get_parent().unwrap().get_allocation().width - evbox.get_allocation().width;

                    let builder: &RefCell<TimelineBuilder> = self_.borrow();
                    builder.borrow().fixed.move_(evbox, cmp::max(cmp::min(rx + x as i32 - builder.borrow().offset, x_max), 0), 50);
                }

                Inhibit(false)
            });
        }
    }
}

pub struct Timeline {
    pub elements: Vec<Box<Component>>,
    pub position: gst::ClockTime,
    pub width: i32,
    pub height: i32,
    pub builder: Rc<RefCell<TimelineBuilder>>,
}

impl Timeline {
    pub fn new(width: i32, height: i32) -> Timeline {
        Timeline {
            elements: vec![],
            position: 0 * gst::MSECOND,
            width: width,
            height: height,
            builder: TimelineBuilder::new(width),
        }
    }

    pub fn new_from_structure(structure: &TimelineStructure) -> Timeline {
        let mut timeline = Timeline::new(structure.size.0, structure.size.1);
        structure.components.iter().for_each(|item| timeline.register(item));
        timeline
    }

    pub fn get_widget(&self) -> gtk::Box {
        (self.builder.borrow() as &RefCell<TimelineBuilder>).borrow().get_widget()
    }

    pub fn register(&mut self, component: &ComponentStructure) {
        let component = Component::new_from_structure(component);

        {
            let time_to_length = |p: gst::ClockTime| p.mseconds().unwrap() as i32;
            let builder = self.builder.clone();
            TimelineBuilder::add_component_widget(builder, &component.name, time_to_length(component.start_time), time_to_length(component.end_time - component.start_time));
        }

        self.elements.push(Box::new(component));
    }

    pub fn seek_to(&mut self, time: gst::ClockTime) {
        self.position = time;
    }

    fn get_current_pixbuf(&self) -> gdk_pixbuf::Pixbuf {
        let pixbuf = unsafe { gdk_pixbuf::Pixbuf::new(0, false, 8, self.width, self.height).unwrap() };

        for p in unsafe { pixbuf.get_pixels().chunks_mut(3) } {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        }

        for elem in &self.elements {
            if let Some(dest) = elem.component.peek(self.position) {
                &dest.composite(
                    &pixbuf, elem.coordinate.0, elem.coordinate.1,
                    cmp::min(dest.get_width(), self.width - elem.coordinate.0),
                    cmp::min(dest.get_height(), self.height - elem.coordinate.1),
                    elem.coordinate.0.into(), elem.coordinate.1.into(), 1f64, 1f64, 0, 255);
            }
        }

        pixbuf
    }

    pub fn renderer(&self, cr: &cairo::Context) -> gtk::Inhibit {
        cr.set_source_pixbuf(&self.get_current_pixbuf(), 0f64, 0f64);
        cr.paint();
        Inhibit(false)
    }

    pub fn write(&mut self, uri: &str, frames: u64, delta: u64) {
        let avi_renderer = AviRenderer::new(uri, self.width as usize, self.height as usize);

        for i in 0..frames {
            if i % 10 == 0 {
                println!("{} / {}", i, frames);
            }
            &avi_renderer.render_step(&self.get_current_pixbuf(), i*delta*gst::MSECOND);
            self.seek_to(i*delta*gst::MSECOND);
        }

        avi_renderer.render_finish();
    }
}

pub struct VideoTestComponent(Component);

impl VideoTestComponent {
    pub fn new(start_time: gst::ClockTime, coordinate: (i32,i32)) -> VideoTestComponent {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
        let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();

        pipeline.add_many(&[&src, &pixbufsink]).unwrap();
        src.link(&pixbufsink).unwrap();

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        VideoTestComponent(Component {
            name: "videotest".to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(pixbufsink),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}

pub struct VideoFileComponent(Component);

impl VideoFileComponent {
    pub fn new(uri: &str, start_time: gst::ClockTime, coordinate: (i32,i32)) -> VideoFileComponent {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("filesrc", None).unwrap();
        let decodebin = gst::ElementFactory::make("decodebin", None).unwrap();
        let queue = gst::ElementFactory::make("queue", None).unwrap();
        let convert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();

        src.set_property("location", &glib::Value::from(uri)).unwrap();

        pipeline.add_many(&[&src, &decodebin, &queue, &convert, &pixbufsink]).unwrap();
        gst::Element::link_many(&[&src, &decodebin]).unwrap();
        gst::Element::link_many(&[&queue, &convert, &pixbufsink]).unwrap();

        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = queue.get_static_pad("sink").unwrap();
            let _ = src_pad.link(&sink_pad);
        });

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        VideoFileComponent(Component {
            name: uri.to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(pixbufsink),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}

impl Peekable for gtk::Image {
    fn get_duration(&self) -> gst::ClockTime {
        100 * gst::MSECOND
    }

    fn peek(&self, _: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.get_pixbuf()
    }
}

pub struct ImageComponent(pub Component);

impl ImageComponent {
    pub fn new(uri: &str, start_time: gst::ClockTime, coordinate: (i32,i32)) -> ImageComponent {
        let image = gtk::Image::new_from_file(uri);

        ImageComponent(Component {
            name: uri.to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(image),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}

impl Peekable for gdk_pixbuf::Pixbuf {
    fn get_duration(&self) -> gst::ClockTime {
        100 * gst::MSECOND
    }

    fn peek(&self, _: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        Some(self.clone())
    }
}

pub struct TextComponent(pub Component);

impl TextComponent {
    pub fn new(label: &str, size: (i32,i32), start_time: gst::ClockTime, coordinate: (i32,i32)) -> TextComponent {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, size.0, size.1).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string("Serif 24"));
        layout.set_markup(format!("<span foreground=\"blue\">{}</span>", label).as_str());
        pangocairo::functions::show_layout(&context, &layout);

        TextComponent(Component {
            name: "text".to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap()),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}


use std::cmp;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;

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

pub mod widget;
use widget::*;

pub mod component;
use component::*;

pub struct Timeline {
    pub elements: Vec<Box<Component>>,
    pub position: gst::ClockTime,
    pub width: i32,
    pub height: i32,
    pub builder: Rc<RefCell<TimelineWidget>>,
    pub canvas: gtk::DrawingArea,
}

impl Timeline {
    pub fn new(width: i32, height: i32) -> Timeline {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(640, 480);

        Timeline {
            elements: vec![],
            position: 0 * gst::MSECOND,
            width: width,
            height: height,
            builder: TimelineWidget::new(width),
            canvas: canvas,
        }
    }

    pub fn setup(self_: Rc<RefCell<Timeline>>) {
        let timeline: &Timeline = &self_.as_ref().borrow();

        {
            let self_ = self_.clone();
            timeline.builder.as_ref().borrow().connect_button_press_event(move |event| {
                let (x,_) = event.get_position();
                self_.borrow_mut().seek_to(x as u64 * gst::MSECOND);

                let timeline = &self_.as_ref().borrow();
                timeline.queue_draw();

                Inhibit(false)
            });
        }

        {
            let self_ = self_.clone();
            timeline.builder.as_ref().borrow().tracker_connect_draw(move |cr| {
                cr.set_source_rgb(200f64, 0f64, 0f64);

                let timeline: &RefCell<Timeline> = self_.borrow();
                let timeline: &Timeline = &timeline.borrow();
                cr.move_to(timeline.position.mseconds().unwrap() as f64, 0f64);
                cr.rel_line_to(0f64, 100f64);
                cr.stroke();

                Inhibit(false)
            });
        }

        {
            let self_ = self_.clone();
            timeline.canvas.connect_draw(move |_,cr| {
                let timeline: &RefCell<Timeline> = self_.borrow();
                timeline.borrow_mut().renderer(cr)
            });
        }
    }

    fn queue_draw(&self) {
        self.canvas.queue_draw();

        let builder: &TimelineWidget = &self.builder.as_ref().borrow();
        builder.queue_draw();
    }

    pub fn new_from_structure(structure: &TimelineStructure) -> Timeline {
        let mut timeline = Timeline::new(structure.size.0, structure.size.1);
        structure.components.iter().for_each(|item| timeline.register(item));
        timeline
    }

    pub fn set_pack_start(&self, vbox: &gtk::Box) {
        let builder: &RefCell<TimelineWidget> = self.builder.borrow();
        vbox.pack_start(builder.borrow().to_widget(), true, true, 0);
    }

    pub fn register(&mut self, component: &ComponentStructure) {
        let component = Component::new_from_structure(component);

        {
            let time_to_length = |p: gst::ClockTime| p.mseconds().unwrap() as i32;
            let builder = self.builder.clone();
            TimelineWidget::add_component_widget(builder, &component.name, time_to_length(component.start_time), time_to_length(component.end_time - component.start_time));
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

        for elem in self.elements.iter().filter(|elem| { elem.start_time <= self.position && self.position <= elem.end_time }) {
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

    fn renderer(&self, cr: &cairo::Context) -> gtk::Inhibit {
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



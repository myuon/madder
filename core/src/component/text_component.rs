use std::marker::PhantomData;

extern crate gstreamer as gst;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;
extern crate pangocairo;
extern crate serde;
extern crate serde_json;

use util::serde_impl::SerRGBA;
use serde::*;
use component::attribute::*;
use component::property::*;
use component::interface::*;

pub struct TextComponent {
    component: ComponentProperty,
    geometry: GeometryProperty,
    text_color: gdk::RGBA,
    text_font: String,
    entity: String,
    data: gdk_pixbuf::Pixbuf,
}

impl Serialize for TextComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serde_json::Map::new();
        map.extend(serde_json::to_value(self.component.clone()).unwrap().as_object().unwrap().clone());
        map.extend(serde_json::to_value(self.geometry.clone()).unwrap().as_object().unwrap().clone());
        map.extend(vec![
            ("text_color".to_string(), json!(SerRGBA(self.text_color))),
            ("text_font".to_string(), json!(self.text_font)),
            ("entity".to_string(), json!(self.entity)),
        ]);

        serde_json::Value::Object(map).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TextComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<TextComponent, D::Error> {
        let json: serde_json::Value = Deserialize::deserialize(deserializer)?;

        Ok(TextComponent::new_from_json(json))
    }
}

impl TextComponent {
    pub fn new_from_json(json: serde_json::Value) -> TextComponent {
        let entity = json.as_object().unwrap()["entity"].as_str().unwrap();
        let text_color = json.as_object().unwrap().get("text_color").map_or_else(
            || SerRGBA(gdk::RGBA::white()),
            |v| serde_json::from_value::<SerRGBA>(v.clone()).unwrap()
        ).0;
        let text_font = json.as_object().unwrap().get("text_font").map_or_else(
            || "Serif 24".to_string(),
            |v| serde_json::from_value(v.clone()).unwrap()
        );
        let data = TextComponent::create_data(entity, &text_font, text_color);

        TextComponent {
            component: serde_json::from_value(json.clone()).unwrap(),
            geometry: serde_json::from_value(json.clone()).unwrap(),
            text_color: text_color,
            text_font: text_font,
            entity: entity.to_string(),
            data: data,
        }
    }

    fn create_data(text: &str, font: &str, color: gdk::RGBA) -> gdk_pixbuf::Pixbuf {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 640, 480).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string(font));
        let markup = format!("<span foreground=\"#{:02X}{:02X}{:02X}{:02X}\">{}</span>", (color.red * 255.0) as i32, (color.green * 255.0) as i32, (color.blue * 255.0) as i32, (color.alpha * 255.0) as i32, text);
        layout.set_markup(markup.as_str());
        pangocairo::functions::show_layout(&context, &layout);

        gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap()
    }

    pub fn reload(&mut self) {
        self.data = TextComponent::create_data(&self.entity, &self.text_font, self.text_color);
    }
}

impl Peekable for TextComponent {
    fn get_duration(&self) -> gst::ClockTime {
        self.data.get_duration()
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.data.peek(time)
    }
}

impl AsProperty for TextComponent {
    fn as_component(&self) -> &ComponentProperty {
        &self.component
    }

    fn as_component_mut(&mut self) -> &mut ComponentProperty {
        &mut self.component
    }

    fn as_geometry(&self) -> Option<&GeometryProperty> {
        Some(&self.geometry)
    }

    fn as_geometry_mut(&mut self) -> Option<&mut GeometryProperty> {
        Some(&mut self.geometry)
    }
}

/*
impl ComponentWrapper for TextComponent {
    fn as_component(&self) -> &Component {
        &self.component
    }

    fn as_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn as_value(&self) -> serde_json::Value {
        let mut json = serde_json::to_value(self.as_component()).unwrap();
        let props = {
            let mut props = serde_json::Map::new();
            for (k,v) in self.get_props() {
                props.insert(k, serde_json::to_value(v).unwrap());
            }

            props
        };

        json.as_object_mut().unwrap().insert("prop".to_string(), json!(props));
        json
    }

    fn get_info(&self) -> String {
        format!("text")
    }
}
*/

impl HasPropertyBuilder for TextComponent {
    fn keys(_: PhantomData<Self>) -> Vec<&'static str> {
        vec_add!(ComponentProperty::keys(PhantomData), vec!["entity", "text_color", "text_font"])
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "entity" => AsAttribute::from_document(self.entity.clone()),
            "text_color" => AsAttribute::from_color(self.text_color.clone()),
            "text_font" => AsAttribute::from_font(self.text_font.clone()),
            k if ComponentProperty::keys(PhantomData).contains(&k) => self.component.getter(k),
            k if GeometryProperty::keys(PhantomData).contains(&k) => self.geometry.getter(k),
            _ => unimplemented!(),
        }
    }

    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T) {
        match name {
            "entity" => {
                self.entity = prop.as_document().unwrap();
                self.reload();
            },
            "text_color" => {
                self.text_color = prop.as_color().unwrap();
                self.reload();
            },
            "text_font" => {
                self.text_font = prop.as_font().unwrap();
                self.reload();
            },
            k if ComponentProperty::keys(PhantomData).contains(&k) => self.component.setter(k, prop),
            k if GeometryProperty::keys(PhantomData).contains(&k) => self.geometry.setter(k, prop),
            _ => unimplemented!(),
        }
    }
}



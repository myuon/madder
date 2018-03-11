extern crate gstreamer as gst;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;
extern crate pangocairo;
extern crate serde;
extern crate serde_json;

use component::property::*;
use component::component::*;

#[derive(Deserialize, Debug, Clone)]
struct TextProperty {
    #[serde(default)]
    common: CommonProperty,

    #[serde(serialize_with = "gdk_rgba_serialize")]
    #[serde(deserialize_with = "gdk_rgba_deserialize")]
    #[serde(default = "gdk::RGBA::white")]
    text_color: gdk::RGBA,

    #[serde(default = "default_text_font")]
    text_font: String,

    entity: String,
}

fn default_text_font() -> String {
    "Serif 24".to_string()
}

impl HasProperty for TextProperty {
    fn get_props(&self) -> Properties {
        use Property::*;

        let mut props = self.common.get_props();
        props.push(("entity".to_string(), Document(self.entity.clone())));
        props.push(("text_font".to_string(), Font(self.text_font.clone())));
        props.push(("text_color".to_string(), Color(self.text_color.clone())));
        props
    }

    fn set_prop(&mut self, name: &str, prop: Property) {
        use Property::*;

        match (name, prop) {
            ("entity", Document(doc)) => {
                self.entity = doc;
                unimplemented!();
            },
            ("text_font", Font(font)) => {
                self.text_font = font;
                unimplemented!();
            },
            ("text_color", Color(rgba)) => {
                self.text_color = rgba;
                unimplemented!();
            },
            (x,y) => {
                self.common.set_prop(x,y.clone());
            },
        }
    }
}

pub struct TextComponent {
    component: Component,
    data: gdk_pixbuf::Pixbuf,
    prop: TextProperty,
}

impl TextComponent {
    pub fn new_from_json(json: serde_json::Value) -> TextComponent {
        let common = serde_json::from_value::<CommonProperty>(json.clone()).unwrap();
        let mut prop = serde_json::from_value::<TextProperty>(json.clone()).unwrap();
        prop.common = common;

        TextComponent {
            component: serde_json::from_value(json).unwrap(),
            data: TextComponent::create_data(&prop.entity, &prop.text_font, prop.text_color),
            prop: prop,
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
        self.data = TextComponent::create_data(&self.prop.entity, &self.prop.text_font, self.prop.text_color);
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

impl AsRef<Component> for TextComponent {
    fn as_ref(&self) -> &Component {
        &self.component
    }
}

impl AsMut<Component> for TextComponent {
    fn as_mut(&mut self) -> &mut Component {
        &mut self.component
    }
}

impl ComponentWrapper for TextComponent {
    fn as_value(&self) -> serde_json::Value {
        let mut json = serde_json::to_value(self.as_ref()).unwrap();
        let props = {
            let mut props = serde_json::Map::new();
            for (k,v) in self.prop.get_props() {
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


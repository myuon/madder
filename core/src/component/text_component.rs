extern crate gstreamer as gst;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;
extern crate pangocairo;
extern crate serde;
extern crate serde_json;
extern crate madder_util as util;

use component::attribute::*;
use component::property::*;
use component::component::*;
use util::serde_impl::*;

#[derive(Deserialize, Debug, Clone)]
struct TextProperty {
    #[serde(default)]
    common: CommonProperty,

    #[serde(serialize_with = "SerRGBA::serialize_rgba")]
    #[serde(deserialize_with = "SerRGBA::deserialize_rgba")]
    #[serde(default = "gdk::RGBA::white")]
    text_color: gdk::RGBA,

    #[serde(default = "default_text_font")]
    text_font: String,

    entity: String,
}

impl TextProperty {
    fn from_value(mut json: serde_json::Value) -> TextProperty {
        let json_ = json.clone();
        json.as_object_mut().unwrap().insert("common".to_string(), json_);
        serde_json::from_value(json).unwrap()
    }
}

fn default_text_font() -> String {
    "Serif 24".to_string()
}

pub struct TextComponent {
    component: Component,
    data: gdk_pixbuf::Pixbuf,
    prop: TextProperty,
}

impl TextComponent {
    pub fn new_from_json(json: serde_json::Value) -> TextComponent {
        let prop = TextProperty::from_value(json.as_object().unwrap()["prop"].clone());

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

impl TextComponent {
    fn keys() -> Vec<String> {
        vec_add!(CommonProperty::keys(), strings!["entity"])
    }
}

impl HasProperty for TextComponent {
    fn get_attr(&self, name: &str) -> Attribute {
        use Attribute::*;

        match name {
            "entity" => Document(self.prop.entity.clone()),
            "text_color" => Color(self.prop.text_color.clone()),
            "text_font" => Font(self.prop.text_font.clone()),
            _ => self.prop.common.get_attr(name),
        }
    }

    fn get_attrs(&self) -> Vec<(String, Attribute)> {
        TextComponent::keys().into_iter().map(|s| (s.clone(), self.get_attr(&s))).collect()
    }

    fn set_attr(&mut self, name: &str, prop: Attribute) {
        use Attribute::*;

        match (name, prop) {
            ("entity", Document(doc)) => {
                self.prop.entity = doc;
                self.reload();
            },
            ("text_color", Color(rgba)) => {
                self.prop.text_color = rgba;
                self.reload();
            },
            ("text_font", Font(font)) => {
                self.prop.text_font = font;
                self.reload();
            },
            (name, prop) => self.prop.common.set_attr(name, prop),
        }
    }

    fn set_prop(&mut self, name: &str, prop: serde_json::Value) {
        match name {
            "entity" => {
                self.prop.entity = serde_json::from_value(prop).unwrap();
                self.reload();
            },
            "text_color" => {
                self.prop.text_color = serde_json::from_value::<SerRGBA>(prop).unwrap().0;
                self.reload();
            },
            "text_font" => {
                self.prop.text_font = serde_json::from_value(prop).unwrap();
                self.reload();
            },
            _ => self.prop.common.set_prop(name, prop),
        }
    }
}



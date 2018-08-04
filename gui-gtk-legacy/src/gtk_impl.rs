extern crate serde_json;
extern crate madder_util as util;

use madder_core::*;
use widget::*;

pub fn attribute_to_widget_type(attr: Attribute) -> WidgetType {
    use Attribute::*;

    match attr {
        ReadOnly(s) => WidgetType::Label(s),
        I32(n) => WidgetType::NumberEntry(From::from(n)),
        F64(n) => WidgetType::NumberEntry(serde_json::Number::from_f64(n).unwrap()),
        Usize(n) => WidgetType::NumberEntry(From::from(n)),
        Time(n) => WidgetType::NumberEntry(serde_json::Number::from_f64(n.mseconds().unwrap() as f64).unwrap()),
        Pair(box x, box y) => {
            let widget_x = attribute_to_widget_type(x);
            let widget_y = attribute_to_widget_type(y);

            WidgetType::Expander(
                format!(
                    "({},{})",
                    if let WidgetType::NumberEntry(x) = &widget_x { x.to_string() } else { "-".to_string() },
                    if let WidgetType::NumberEntry(y) = &widget_y { y.to_string() } else { "-".to_string() },
                ),
                Box::new(WidgetType::VBox(vec![
                    widget_x,
                    widget_y,
                ]))
            )
        },
        FilePath(path) => WidgetType::FileChooser(path),
        Document(doc) => WidgetType::TextArea(doc),
        Font(font) => WidgetType::Font(font),
        Color(color) => WidgetType::Color(color),
        Choose(options, index) => WidgetType::Choose(options, index),
        Sequence(seq) => WidgetType::VBox(seq.into_iter().map(attribute_to_widget_type).collect()),
    }
}

pub fn widget_type_to_value(widget_type: WidgetType) -> serde_json::Value {
    match widget_type {
        WidgetType::NumberEntry(label) => json!(label),
        WidgetType::TextEntry(label) => json!(label),
        WidgetType::Choose(_, index) => json!(index),
        WidgetType::Label(_) => unreachable!(),
        WidgetType::Grid(vec) => {
            json!(vec.into_iter().map(|(key,widget_type)| {
                (key, widget_type_to_value(widget_type))
            }).collect::<Vec<_>>())
        },
        WidgetType::VBox(_) => unimplemented!(),
        WidgetType::Expander(_,_) => unimplemented!(),
        WidgetType::FileChooser(_) => unimplemented!(),
        WidgetType::TextArea(doc) => json!(doc),
        WidgetType::Font(font) => json!(font),
        WidgetType::Color(color) => json!(util::serde_impl::SerRGBA(color)),
    }
}


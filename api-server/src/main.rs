extern crate serde_json;
extern crate madder_core;
extern crate madder_util as util;
extern crate ws;
use madder_core::*;
use std::rc::Rc;
use std::cell::RefCell;

fn req(app: Rc<RefCell<Madder>>, msg: ws::Message) -> Result<ws::Message, String> {
    let msg_text = msg.as_text().unwrap();
    let req = serde_json::from_str::<Request>(msg_text).map_err(|err| err.to_string())?;

    if req.path == "/screen" {
        Ok(ws::Message::Binary(app.borrow().get_pixbuf(serde_json::from_value::<util::serde_impl::SerTime>(req.entity).unwrap().0).save_to_bufferv("png", &[]).unwrap()))
    } else {
        let result: serde_json::Value = app.borrow_mut().req(req)?;
        serde_json::to_string(&result).map_err(|err| err.to_string()).map(ws::Message::Text)
    }
}

fn main() {
    let app = Madder::new();
    let app_ref = Rc::new(RefCell::new(app));

    println!("listening on localhost:3000...");
    ws::listen("localhost:3000", |socket| {
        let app_ref = app_ref.clone();

        move |msg| {
            match req(app_ref.clone(), msg) {
                Ok(response) => socket.send(response),
                Err(err) => socket.send(err),
            }
        }
    }).unwrap();
}

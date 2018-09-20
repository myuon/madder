extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate madder_core;
extern crate ws;
use madder_core::*;
use madder_core::util;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Deserialize)]
struct WriteEntity {
    uri: String,
    frames: i32,
    delta: u64,
}

fn req(app: Rc<RefCell<Madder>>, msg: ws::Message, socket: &ws::Sender) -> Result<ws::Message, String> {
    let msg_text = msg.as_text().unwrap();
    let req = serde_json::from_str::<Request>(msg_text).map_err(|err| err.to_string())?;

    if req.path == "/screen" {
        let stream = app.borrow().get_pixbuf(serde_json::from_value::<util::SerTime>(req.entity).unwrap().0).save_to_bufferv("png", &[]).unwrap();
        Ok(ws::Message::Binary(stream))
    } else if req.path == "/write" {
        println!("write");
        // I know this is a bad way to block main thread, but ...
        let write_entity = serde_json::from_value::<WriteEntity>(req.entity).unwrap();
        app.borrow_mut().render_init(&write_entity.uri, write_entity.frames, write_entity.delta);

        loop {
            let (cont, frac) = app.borrow_mut().render_next();
            socket.send(ws::Message::Text(format!("{}", frac))).unwrap();

            if cont == false {
                break;
            }
        }

        Ok(ws::Message::Text("".to_string()))
    } else {
        let result: serde_json::Value = app.borrow_mut().req(req)?;
        serde_json::to_string(&result).map_err(|err| err.to_string()).map(ws::Message::Text)
    }
}

fn main() {
    madder_core::init();

    let app = Madder::new();
    let app_ref = Rc::new(RefCell::new(app));

    println!("listening on localhost:3000...");
    ws::listen("localhost:3000", |socket| {
        let app_ref = app_ref.clone();

        move |msg| {
            match req(app_ref.clone(), msg, &socket) {
                Ok(response) => socket.send(response),
                Err(err) => socket.send(err),
            }
        }
    }).unwrap();
}

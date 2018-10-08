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
    length: i32,
    fps: i32,
}

#[derive(Serialize)]
struct Response {
    status: i16,
    body: String,
}

impl Response {
    fn to_message(&self) -> ws::Message {
        ws::Message::Text(serde_json::to_string(&self).unwrap())
    }
}

fn req(app: Rc<RefCell<Madder>>, msg: ws::Message, socket: &ws::Sender) -> Result<(), ws::Error> {
    let msg_text = msg.as_text().unwrap();
    let req = match serde_json::from_str::<Request>(msg_text) {
        Ok(req) => req,
        Err(err) => {
            let response = Response {
                status: 400,
                body: err.to_string(),
            };

            return socket.send(response.to_message());
        },
    };

    if req.path == "/screen" {
        let stream = app.borrow().get_pixbuf(serde_json::from_value::<util::SerTime>(req.entity).unwrap().0).save_to_bufferv("png", &[]).unwrap();

        let response = Response {
            status: 200,
            body: "".to_string(),
        };
        socket.send(response.to_message()).and_then(|()| socket.send(ws::Message::Binary(stream)))
    } else if req.path == "/write" {
        // I know this is a bad way to block main thread, but ...
        let write_entity = serde_json::from_value::<WriteEntity>(req.entity).unwrap();
        app.borrow_mut().start_render(&write_entity.uri, write_entity.length * write_entity.fps, write_entity.fps);

        let response = Response {
            status: 200,
            body: "{}".to_string(),
        };
        socket.send(response.to_message())
    } else {
        let response = match app.borrow_mut().req(req) {
            Ok(result) => {
                Response {
                    status: 200,
                    body: serde_json::to_string(&result).unwrap(),
                }
            },
            Err(err) => {
                Response {
                    status: 500,
                    body: err,
                }
            },
        };

        socket.send(response.to_message())
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
            req(app_ref.clone(), msg, &socket)
        }
    }).unwrap();
}

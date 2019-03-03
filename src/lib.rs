use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use futures::{future, Future};
//use wasm_bindgen::convert::ReturnWasmAbi;
use js_sys::{Array, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, Document, Element, HtmlDivElement, HtmlImageElement, Request, RequestInit,
    RequestMode, Response, SvgsvgElement,
};

use js_utils::*;
use zoom::*;

mod js_utils;
mod zoom;

#[wasm_bindgen]
pub fn init() -> Result<Promise, JsValue> {
    console_error_panic_hook::set_once();

    // grab all the
    let zoom_nodes = document()
        .query_selector_all("[data-archizoom]")?
        .safe_filter::<HtmlImageElement>();

    let result_futures = Array::new();
    for node in zoom_nodes.into_iter() {
        match new_archizoom(node) {
            Ok(p) => {
                result_futures.push(&p);
            }
            Err(e) => console::error_2(&"Couldn't initialize archizoom".into(), &e),
        }
    }

    Ok(Promise::all(&result_futures))
}

fn new_archizoom(img: HtmlImageElement) -> Result<Promise, JsValue> {
    let src = img.src();
    let parent = img
        .parent_element()
        .ok_or::<JsValue>("The image element must have a parent".into())?;

    let mut opts = RequestInit::new();
    opts.method("GET");

    let request = Request::new_with_str_and_init(&src, &opts)?;

    let request_promise = window().fetch_with_request(&request);

    let future = JsFuture::from(request_promise)
        .and_then(|resp_value| {
            // grab the text from our response
            resp_value
                .dyn_into::<Response>()
                .and_then(|response| response.text())
        })
        .and_then(|text: Promise| {
            // Convert the response promise into a future
            JsFuture::from(text)
        })
        .and_then(move |text_value| {
            let text = text_value.as_string();

            // create a new container
            let container = document()
                .safe_create_element::<HtmlDivElement>("div")
                .unwrap();

            container.set_inner_html(&text.unwrap());

            // find the embedded SvgsvgElement
            let svg = container
                .first_element_child()
                .ok_or::<JsValue>("The image element must have a parent".into())
                .and_then(|child| child.dyn_into::<SvgsvgElement>().map_err(|e| e.into()))?;

            ArchiZoom::new(svg).and_then(|az| {
                parent
                    .replace_child(&container, &img)
                    .map(|_| JsValue::from(az))
            })
        });

    // Convert this Rust `Future` back into a JS `Promise`.
    Ok(future_to_promise(future))
}

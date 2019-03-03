use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, HtmlDivElement, SvgElement, SvgsvgElement};

use svg_view_controller::SvgViewController;

use crate::js_utils::*;

mod svg_view_controller;

//pub fn new_svg() -> Result<(), JsValue> {
//
//}

#[wasm_bindgen]
pub struct ArchiZoom {
    svg: SvgsvgElement,

    _view_controller: Rc<RefCell<SvgViewController>>,
}

impl ArchiZoom {
    pub fn new(svg: SvgsvgElement) -> Result<ArchiZoom, JsValue> {
        svg.set_attribute("viewBox", "0 0 200 200");

        let archizoom = ArchiZoom {
            _view_controller: SvgViewController::new(&svg),
            svg,
        };

        Ok(archizoom)
    }
}

//impl Drop for ArchiZoom {
//    fn drop(&mut self) {
//        console::log_1(&"dropped archizoom".into());
//    }
//}

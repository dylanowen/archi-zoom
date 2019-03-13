use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::{console, SvgaElement, SvgsvgElement};

use svg_view_controller::SvgViewController;

use crate::events::EventSource;
use crate::js_utils::*;
use crate::zoom::matrix::{Matrix2D, Rect};
use crate::zoom::svg_view_controller::ViewUpdateEvent;
use crate::PREFIX_ALIAS;

mod matrix;
mod svg_view_controller;

#[wasm_bindgen]
pub struct ArchiZoom {
    _svg: SvgsvgElement,
    zoom_elements: Vec<ZoomElement>,
    view_controller: Rc<RefCell<SvgViewController>>,
}

struct ZoomElement {
    _link: String,
    link_element: SvgaElement,
}

static X_LINK_NS: &str = "http://www.w3.org/1999/xlink";
static VIEW_THRESHOLD: f32 = 0.45;

impl ArchiZoom {
    pub fn new(svg: SvgsvgElement) -> Result<Rc<RefCell<ArchiZoom>>, JsValue> {
        let zoom_areas = svg
            .query_selector_all(&format!("[*|href*=\"#{}:link\"]", PREFIX_ALIAS))?
            .safe_filter::<SvgaElement>()
            .into_iter()
            .map(|link_element| {
                let zoom_element = ZoomElement {
                    _link: link_element.href().base_val(),
                    link_element,
                };

                // TODO we really need to actually just replace this with some other non-clickable thing
                zoom_element
                    .link_element
                    .set_attribute_ns(Some(X_LINK_NS), "href", "#")
                    .expect("We should always be able to clear the xlink:href attribute");

                zoom_element
            })
            .collect();

        let view_controller = SvgViewController::new(&svg)?;

        let archizoom = Rc::new(RefCell::new(ArchiZoom {
            view_controller,
            zoom_elements: zoom_areas,
            _svg: svg,
        }));

        let callback_ref = Rc::downgrade(&archizoom);

        archizoom
            .borrow()
            .view_controller
            .borrow_mut()
            .register_listener(move |e: &ViewUpdateEvent| {
                if let Some(real_ref) = callback_ref.upgrade() {
                    real_ref.borrow().view_update(e)
                }
            });

        Ok(archizoom)
    }

    fn view_update(&self, event: &ViewUpdateEvent) {
        let viewport = event.viewport();
        for zoom_element in self.zoom_elements.iter() {
            if let Some(element_rect) = zoom_element.element_rect() {
                #[inline]
                fn overlap(a_left: f32, a_right: f32, b_left: f32, b_right: f32) -> f32 {
                    a_right.min(b_right) - a_left.max(b_left)
                }

                let horizontal_overlap = overlap(
                    viewport.left(),
                    viewport.right(),
                    element_rect.left(),
                    element_rect.right(),
                );
                let vertical_overlap = overlap(
                    viewport.top(),
                    viewport.bottom(),
                    element_rect.top(),
                    element_rect.bottom(),
                );

                let total_area = viewport.area();
                let viewable_area = horizontal_overlap * vertical_overlap;
                let area_percentage = viewable_area / total_area;

                if area_percentage >= VIEW_THRESHOLD {
                    console::log_1(&"in view".into());
                }
            }
        }
    }
}

impl ZoomElement {
    /// Gets the element Rect in Svg Viewport Coordinates
    fn element_rect(&self) -> Option<Rect> {
        self.link_element.get_b_box().ok().and_then(|element_box| {
            self.link_element
                .get_screen_ctm()
                .map(|m| Rect::from_svg(&element_box).matrix_transform(&Matrix2D::from_js(&m)))
        })
    }
}

impl Drop for ArchiZoom {
    fn drop(&mut self) {
        console::log_1(&"dropped ArchiZoom".into());
    }
}

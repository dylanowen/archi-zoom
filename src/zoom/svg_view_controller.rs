use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{
    console, Event, MouseEvent, PointerEvent, SvgPoint, SvgsvgElement, TouchEvent, WheelEvent,
};

use crate::js_utils::{SafeSelfClosure, SelfClosure};

pub struct SvgViewController {
    svg: SvgsvgElement,

    is_pointer_down: bool,
    pointer_origin: SvgPoint,

    events: Vec<Box<SelfClosure>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Position(i32, i32);

impl SvgViewController {
    pub fn new(svg: &SvgsvgElement) -> Rc<RefCell<SvgViewController>> {
        let view_controller = Rc::new(RefCell::new(SvgViewController {
            pointer_origin: svg.create_svg_point(),
            svg: svg.clone(),
            is_pointer_down: false,
            events: vec![],
        }));

        register_drag_events(&view_controller);
        register_scroll_events(&view_controller);

        view_controller
    }

    fn on_pointer_down(&mut self, position: Position, _event: Event) {
        if let Some(point) = self.get_point(&position) {
            self.is_pointer_down = true;

            self.pointer_origin = point;
        }
    }

    fn on_pointer_move(&mut self, position: Position, event: Event) {
        if self.is_pointer_down {
            event.prevent_default();

            if let Some(point) = self.get_point(&position) {
                if let Some(view_box) = self.svg.view_box().base_val() {
                    let delta_x = point.x() - self.pointer_origin.x();
                    let delta_y = point.y() - self.pointer_origin.y();

                    view_box.set_x(view_box.x() - delta_x);
                    view_box.set_y(view_box.y() - delta_y);
                }
            }
        }
    }

    fn on_pointer_up(&mut self, _event: Event) {
        self.is_pointer_down = false;
    }

    fn on_scroll(&mut self, delta_y: f32, event: Event) {
        event.prevent_default();

        console::log_1(&delta_y.to_string().into());

        if let Some(view_box) = self.svg.view_box().base_val() {
            view_box.set_width(view_box.width() + delta_y);
            view_box.set_height(view_box.height() + delta_y);
        }
    }

    fn get_point(&mut self, position: &Position) -> Option<SvgPoint> {
        let point = self.svg.create_svg_point();

        point.set_x(position.0 as f32);
        point.set_y(position.1 as f32);

        if let Some(svg_matrix) = self.svg.get_screen_ctm() {
            if let Ok(inverted_svg_matrix) = svg_matrix.inverse() {
                return Some(point.matrix_transform(&inverted_svg_matrix));
            }
        }

        return None;
    }
}

impl Drop for SvgViewController {
    fn drop(&mut self) {
        for closure in self.events.iter_mut() {
            closure.remove(&self.svg)
        }
    }
}

fn register_drag_events(svg_ref: &Rc<RefCell<SvgViewController>>) {
    // check if pointer events are supported
    let mut events = match PointerEvent::new("pointerdown") {
        Ok(_) => {
            // pointers are supported
            vec![
                svg_ref.new_self_closure(
                    &"pointerdown",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: PointerEvent| {
                        svg_ref.borrow_mut().on_pointer_down(
                            Position(event.client_x(), event.client_y()),
                            event.into(),
                        );
                    },
                ),
                svg_ref.new_self_closure(
                    &"pointermove",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: PointerEvent| {
                        svg_ref.borrow_mut().on_pointer_move(
                            Position(event.client_x(), event.client_y()),
                            event.into(),
                        );
                    },
                ),
                svg_ref.new_self_closure(
                    &"pointerup",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: PointerEvent| {
                        svg_ref.borrow_mut().on_pointer_up(event.into());
                    },
                ),
                svg_ref.new_self_closure(
                    &"pointerleave",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: PointerEvent| {
                        svg_ref.borrow_mut().on_pointer_up(event.into());
                    },
                ),
            ]
        }
        Err(_) => {
            fn touch_position(event: &TouchEvent) -> Position {
                if let Some(ref touch) = event.touches().get(0) {
                    Position(touch.client_x(), touch.client_y())
                } else {
                    Position(0, 0)
                }
            }

            // no pointer support, so use something else
            vec![
                svg_ref.new_self_closure(
                    &"mousedown",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: MouseEvent| {
                        svg_ref.borrow_mut().on_pointer_down(
                            Position(event.client_x(), event.client_y()),
                            event.into(),
                        );
                    },
                ),
                svg_ref.new_self_closure(
                    &"mousemove",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: MouseEvent| {
                        svg_ref.borrow_mut().on_pointer_move(
                            Position(event.client_x(), event.client_y()),
                            event.into(),
                        );
                    },
                ),
                svg_ref.new_self_closure(
                    &"mouseup",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: MouseEvent| {
                        svg_ref.borrow_mut().on_pointer_up(event.into());
                    },
                ),
                svg_ref.new_self_closure(
                    &"mouseleave",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: MouseEvent| {
                        svg_ref.borrow_mut().on_pointer_up(event.into());
                    },
                ),
                svg_ref.new_self_closure(
                    &"touchstart",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: TouchEvent| {
                        svg_ref
                            .borrow_mut()
                            .on_pointer_down(touch_position(&event), event.into());
                    },
                ),
                svg_ref.new_self_closure(
                    &"touchmove",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: TouchEvent| {
                        svg_ref
                            .borrow_mut()
                            .on_pointer_move(touch_position(&event), event.into());
                    },
                ),
                svg_ref.new_self_closure(
                    &"touchend",
                    &svg_ref.borrow().svg,
                    |svg_ref, event: TouchEvent| {
                        svg_ref.borrow_mut().on_pointer_up(event.into());
                    },
                ),
            ]
        }
    };

    svg_ref.borrow_mut().events.append(&mut events);
}

fn register_scroll_events(svg_ref: &Rc<RefCell<SvgViewController>>) {
    let event = svg_ref.new_self_closure(
        &"wheel",
        &svg_ref.borrow().svg,
        |svg_ref, event: WheelEvent| {
            svg_ref
                .borrow_mut()
                .on_scroll(event.delta_y() as f32, event.into());
        },
    );

    svg_ref.borrow_mut().events.push(event);
}

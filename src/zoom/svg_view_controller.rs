use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::JsValue;
use web_sys::{Event, MouseEvent, PointerEvent, SvgPoint, SvgsvgElement, TouchEvent, WheelEvent};

use crate::events::{EventListener, EventSource};
use crate::js_utils::{EnhancedEventTarget, JsEventListener};
use crate::zoom::matrix::{Point2D, Rect};

pub struct SvgViewController {
    svg: SvgsvgElement,

    is_pointer_down: bool,
    pointer_origin: SvgPoint,

    listeners: Vec<Box<EventListener<ViewUpdateEvent>>>,
    event_listeners: Vec<Box<JsEventListener>>,
}

#[derive(Debug)]
pub struct ViewUpdateEvent {
    /// The coordinates in Svg Viewport Coordinates in pixels
    viewport: Rect,
}

static ZOOM_FACTOR: f32 = 0.003;

impl SvgViewController {
    pub fn new(svg: &SvgsvgElement) -> Result<Rc<RefCell<SvgViewController>>, JsValue> {
        let view_controller = Rc::new(RefCell::new(SvgViewController {
            pointer_origin: svg.create_svg_point(),
            svg: svg.clone(),
            is_pointer_down: false,
            listeners: vec![],
            event_listeners: vec![],
        }));

        get_drag_events(&view_controller)?;
        register_scroll_events(&view_controller)?;

        Ok(view_controller)
    }

    fn on_pointer_down(&mut self, position: Point2D, _event: Event) {
        if let Some(point) = self.get_point(&position) {
            self.is_pointer_down = true;

            self.pointer_origin = point;
        }
    }

    fn on_pointer_move(&self, position: Point2D, event: Event) {
        if self.is_pointer_down {
            event.prevent_default();

            if let Some(point) = self.get_point(&position) {
                if let Some(view_box) = self.svg.view_box().base_val() {
                    let delta_x = point.x() - self.pointer_origin.x();
                    let delta_y = point.y() - self.pointer_origin.y();

                    view_box.set_x(view_box.x() - delta_x);
                    view_box.set_y(view_box.y() - delta_y);

                    self.dispatch_event();
                }
            }
        }
    }

    fn on_pointer_up(&mut self, _event: Event) {
        self.is_pointer_down = false;
    }

    fn on_scroll(&self, delta_y: f32, _position: Point2D, event: Event) {
        event.prevent_default();

        if let Some(view_box) = self.svg.view_box().base_val() {
            let delta_width = view_box.width() * (delta_y * ZOOM_FACTOR);
            let delta_height = view_box.height() * (delta_y * ZOOM_FACTOR);

            view_box.set_width(view_box.width() + delta_width);
            view_box.set_height(view_box.height() + delta_height);
            view_box.set_x(view_box.x() - (delta_width / 2.0));
            view_box.set_y(view_box.y() - (delta_height / 2.0));

            self.dispatch_event();
        }
    }

    fn dispatch_event(&self) {
        let client_rect = self.svg.get_bounding_client_rect();
        let viewport = Rect::new(
            Point2D { x: 0.0, y: 0.0 },
            Point2D {
                x: client_rect.width() as f32,
                y: client_rect.height() as f32,
            },
        );

        let event = ViewUpdateEvent { viewport };

        for listener in self.listeners.iter() {
            listener.receive(&event);
        }
    }

    fn get_point(&self, position: &Point2D) -> Option<SvgPoint> {
        let point = self.svg.create_svg_point();

        point.set_x(position.x);
        point.set_y(position.y);

        if let Some(svg_matrix) = self.svg.get_screen_ctm() {
            if let Ok(inverted_svg_matrix) = svg_matrix.inverse() {
                return Some(point.matrix_transform(&inverted_svg_matrix));
            }
        }

        return None;
    }
}

impl EventSource<ViewUpdateEvent> for SvgViewController {
    fn register_listener<T: EventListener<ViewUpdateEvent> + 'static>(&mut self, callback: T) {
        self.listeners.push(Box::new(callback));
    }
}

impl ViewUpdateEvent {
    #[inline]
    pub fn viewport(&self) -> &Rect {
        &self.viewport
    }
}

fn get_drag_events(view_controller_ref: &Rc<RefCell<SvgViewController>>) -> Result<(), JsValue> {
    // check if pointer events are supported
    let mut events = match PointerEvent::new("pointerdown") {
        Ok(_) => {
            // pointers are supported
            vec![
                add_svg_event(
                    view_controller_ref,
                    &"pointerdown",
                    |controller_ref, event: PointerEvent| {
                        controller_ref.borrow_mut().on_pointer_down(
                            Point2D::new(event.client_x() as f32, event.client_y() as f32),
                            event.into(),
                        );
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"pointermove",
                    |controller_ref, event: PointerEvent| {
                        controller_ref.borrow().on_pointer_move(
                            Point2D::new(event.client_x() as f32, event.client_y() as f32),
                            event.into(),
                        );
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"pointerup",
                    |controller_ref, event: PointerEvent| {
                        controller_ref.borrow_mut().on_pointer_up(event.into());
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"pointerleave",
                    |controller_ref, event: PointerEvent| {
                        controller_ref.borrow_mut().on_pointer_up(event.into());
                    },
                )?,
            ]
        }
        Err(_) => {
            fn touch_position(event: &TouchEvent) -> Point2D {
                if let Some(ref touch) = event.touches().get(0) {
                    Point2D::new(touch.client_x() as f32, touch.client_y() as f32)
                } else {
                    Point2D::new(0.0, 0.0)
                }
            }

            // no pointer support, so use something else
            vec![
                add_svg_event(
                    view_controller_ref,
                    &"mousedown",
                    |controller_ref, event: MouseEvent| {
                        controller_ref.borrow_mut().on_pointer_down(
                            Point2D::new(event.client_x() as f32, event.client_y() as f32),
                            event.into(),
                        );
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"mousemove",
                    |controller_ref, event: MouseEvent| {
                        controller_ref.borrow().on_pointer_move(
                            Point2D::new(event.client_x() as f32, event.client_y() as f32),
                            event.into(),
                        );
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"mouseup",
                    |controller_ref, event: MouseEvent| {
                        controller_ref.borrow_mut().on_pointer_up(event.into());
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"mouseleave",
                    |controller_ref, event: MouseEvent| {
                        controller_ref.borrow_mut().on_pointer_up(event.into());
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"touchstart",
                    |controller_ref, event: TouchEvent| {
                        controller_ref
                            .borrow_mut()
                            .on_pointer_down(touch_position(&event), event.into());
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"touchmove",
                    |controller_ref, event: TouchEvent| {
                        controller_ref
                            .borrow()
                            .on_pointer_move(touch_position(&event), event.into());
                    },
                )?,
                add_svg_event(
                    view_controller_ref,
                    &"touchend",
                    |controller_ref, event: TouchEvent| {
                        controller_ref.borrow_mut().on_pointer_up(event.into());
                    },
                )?,
            ]
        }
    };

    view_controller_ref
        .borrow_mut()
        .event_listeners
        .append(&mut events);

    Ok(())
}

fn register_scroll_events(
    view_controller_ref: &Rc<RefCell<SvgViewController>>,
) -> Result<(), JsValue> {
    let event = add_svg_event(
        view_controller_ref,
        &"wheel",
        |controller_ref, event: WheelEvent| {
            controller_ref.borrow().on_scroll(
                event.delta_y() as f32,
                Point2D::new(event.client_x() as f32, event.client_y() as f32),
                event.into(),
            );
        },
    )?;

    view_controller_ref.borrow_mut().event_listeners.push(event);

    Ok(())
}

fn add_svg_event<C, E>(
    controller_ref: &Rc<RefCell<SvgViewController>>,
    event_type: &str,
    callback: C,
) -> Result<Box<JsEventListener>, JsValue>
where
    C: Fn(Rc<RefCell<SvgViewController>>, E) + 'static,
    E: FromWasmAbi + 'static,
{
    let svg = &controller_ref.borrow().svg;

    let weak_ref = Rc::downgrade(controller_ref);
    svg.new_event_listener(event_type, move |event: E| {
        if let Some(real_ref) = weak_ref.upgrade() {
            callback(real_ref, event)
        }
    })
}

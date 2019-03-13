use wasm_bindgen::closure::Closure;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{console, Document, Element, EventTarget, NodeList, Window};

pub trait EnhancedDocument {
    fn safe_get_by_id<T: JsCast>(&self, id: &str) -> Option<T>;

    fn safe_create_element<T: JsCast>(&self, id: &str) -> Option<T>;

    fn safe_create_element_ns<T: JsCast>(&self, namespace: Option<&str>, id: &str) -> Option<T>;
}

impl EnhancedDocument for Document {
    fn safe_get_by_id<T: JsCast>(&self, id: &str) -> Option<T> {
        match self.get_element_by_id(id) {
            Some(element) => element.safe_cast::<T>(),
            None => {
                console::error_1(&format!("Couldn't find element with id: {}", id).into());

                None
            }
        }
    }

    fn safe_create_element<T: JsCast>(&self, id: &str) -> Option<T> {
        match document().create_element(id) {
            Ok(element) => element.safe_cast::<T>(),
            Err(error) => {
                console::error_2(
                    &format!("Couldn't create an element with id: {}", id).into(),
                    &error,
                );

                None
            }
        }
    }

    fn safe_create_element_ns<T: JsCast>(&self, namespace: Option<&str>, id: &str) -> Option<T> {
        match document().create_element_ns(namespace, id) {
            Ok(element) => element.safe_cast::<T>(),
            Err(error) => {
                console::error_2(
                    &format!("Couldn't create an element with id: {}", id).into(),
                    &error,
                );

                None
            }
        }
    }
}

pub trait EnhancedElement {
    fn safe_cast<T: JsCast>(self) -> Option<T>;
}

impl EnhancedElement for Element {
    fn safe_cast<T: JsCast>(self) -> Option<T> {
        match self.dyn_into::<T>() {
            Ok(success) => Some(success),
            Err(error) => {
                console::error_2(
                    &error,
                    &format!("Can't be cast because it's a {}", error.tag_name()).into(),
                );

                None
            }
        }
    }
}

pub trait EnhancedNodeList {
    fn safe_filter<T: JsCast>(self) -> Vec<T>;
}

impl EnhancedNodeList for NodeList {
    fn safe_filter<T: JsCast>(self) -> Vec<T> {
        let mut valid_nodes = vec![];

        for i in 0..self.length() {
            match self.get(i).and_then(|node| node.dyn_into::<T>().ok()) {
                Some(t) => valid_nodes.push(t),
                None => (),
            }
        }

        valid_nodes
    }
}

pub trait JsEventListener {
    fn remove(&mut self);
}

struct JsEventListenerImpl<T: ?Sized> {
    event_type: String,
    target: EventTarget,
    closure: Option<Closure<T>>,
}

impl<T: ?Sized> JsEventListener for JsEventListenerImpl<T> {
    fn remove(&mut self) {
        if let Some(ref closure) = self.closure {
            match self.target.remove_event_listener_with_callback(
                &self.event_type,
                closure.as_ref().unchecked_ref(),
            ) {
                Ok(_) => (),
                Err(error) => console::warn_2(&"Failed to remove event handler".into(), &error),
            }
        }

        self.closure = None;
    }
}

impl<T: ?Sized> Drop for JsEventListenerImpl<T> {
    fn drop(&mut self) {
        console::log_1(&"Dropping JsEventListener".into());
        self.remove();
    }
}

pub trait EnhancedEventTarget {
    fn new_event_listener<C, E>(
        &self,
        event_type: &str,
        callback: C,
    ) -> Result<Box<JsEventListener>, JsValue>
    where
        C: Fn(E) + 'static,
        E: FromWasmAbi + 'static;
}

impl EnhancedEventTarget for EventTarget {
    fn new_event_listener<C, E>(
        &self,
        event_type: &str,
        callback: C,
    ) -> Result<Box<JsEventListener>, JsValue>
    where
        C: Fn(E) + 'static,
        E: FromWasmAbi + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<Fn(E)>);

        self.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
            .map(|_| -> Box<JsEventListener> {
                Box::new(JsEventListenerImpl {
                    event_type: event_type.to_string(),
                    target: self.clone(),
                    closure: Some(closure),
                })
            })
    }
}

pub fn window() -> Window {
    web_sys::window().expect("Missing window")
}

pub fn document() -> Document {
    window().document().expect("Missing document")
}

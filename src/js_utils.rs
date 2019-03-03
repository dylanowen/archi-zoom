use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
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

pub trait SafeSelfClosure {
    type Inner;

    fn new_self_closure<C, E>(
        &self,
        _type: &str,
        event_target: &EventTarget,
        closure: C,
    ) -> Box<dyn SelfClosure>
    where
        C: Fn(Rc<Self::Inner>, E) + 'static,
        E: FromWasmAbi + 'static;
}

impl<V> SafeSelfClosure for Rc<RefCell<V>>
where
    V: 'static,
{
    type Inner = RefCell<V>;

    fn new_self_closure<C, E>(
        &self,
        _type: &str,
        event_target: &EventTarget,
        closure: C,
    ) -> Box<dyn SelfClosure>
    where
        C: Fn(Rc<Self::Inner>, E) + 'static,
        E: FromWasmAbi + 'static,
    {
        let weak_self = Rc::downgrade(self);

        let closure = Closure::wrap(Box::new(move |event| match weak_self.upgrade() {
            Some(real_self) => closure(real_self, event),
            None => (),
        }) as Box<Fn(E)>);

        match event_target.add_event_listener_with_callback(_type, closure.as_ref().unchecked_ref())
        {
            Ok(_) => (),
            Err(error) => console::error_2(&"Failed to add event handler".into(), &error),
        }

        Box::new(SelfClosureImpl {
            _type: _type.to_string(),
            closure: Some(closure),
        })
    }
}

pub trait SelfClosure {
    fn remove(&mut self, event_target: &EventTarget);
}

struct SelfClosureImpl<T: ?Sized> {
    _type: String,
    closure: Option<Closure<T>>,
}

impl<T: ?Sized> SelfClosure for SelfClosureImpl<T> {
    fn remove(&mut self, event_target: &EventTarget) {
        if let Some(ref closure) = self.closure {
            match event_target.remove_event_listener_with_callback(
                self._type.as_str(),
                closure.as_ref().unchecked_ref(),
            ) {
                Ok(_) => (),
                Err(error) => console::warn_2(&"Failed to remove event handler".into(), &error),
            }
        }

        self.closure = None;
    }
}

impl<T: ?Sized> Drop for SelfClosureImpl<T> {
    fn drop(&mut self) {
        console::log_1(&"dropping".into());

        if self.closure.is_some() {
            console::error_1(&"Dropping a registered closure: The event will throw an exception and can't be cleaned up".into());
        }
    }
}

//pub fn benchmark<O, F>(description: &str, mut f: F) -> O
//where
//    F: FnMut() -> O,
//{
//    match window().performance() {
//        Some(perf) => {
//            let start = perf.now();
//            let result = f();
//            let end = perf.now();
//            console::log_1(&format!("{}: {}ms", description, (end - start)).into());
//
//            result
//        }
//        None => {
//            console::warn_1(&"No performance object available".into());
//            f()
//        }
//    }
//}

pub fn window() -> Window {
    web_sys::window().expect("Missing window")
}

pub fn document() -> Document {
    window().document().expect("Missing document")
}

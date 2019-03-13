pub trait EventListener<E> {
    fn receive(&self, event: &E);
}

impl<E, F: Fn(&E)> EventListener<E> for F {
    fn receive(&self, event: &E) {
        self(event)
    }
}

impl<E> EventListener<E> for Fn(&E) {
    fn receive(&self, event: &E) {
        self(event)
    }
}

pub trait EventSource<E> {
    fn register_listener<T: EventListener<E> + 'static>(&mut self, listener: T);
}

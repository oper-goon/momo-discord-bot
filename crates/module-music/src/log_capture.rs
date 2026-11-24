//! In-memory subscriber for checking the production log boundary in tests.
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
};

#[derive(Clone, Default)]
pub(crate) struct Capture {
    lines: Arc<Mutex<Vec<String>>>,
    ids: Arc<AtomicU64>,
}
impl Capture {
    pub(crate) fn snapshot(&self) -> String {
        self.lines.lock().unwrap().join("\n")
    }
    fn push(&self, line: String) {
        self.lines.lock().unwrap().push(line);
    }
}
struct Fields<'a>(&'a mut String);
impl Visit for Fields<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        write!(self.0, " {}={value:?}", field.name()).unwrap();
    }
}
impl Subscriber for Capture {
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
        true
    }
    fn new_span(&self, attrs: &Attributes<'_>) -> Id {
        let mut line = format!("span {}", attrs.metadata().name());
        attrs.record(&mut Fields(&mut line));
        self.push(line);
        Id::from_u64(self.ids.fetch_add(1, Ordering::Relaxed) + 1)
    }
    fn record(&self, _: &Id, values: &Record<'_>) {
        let mut line = String::new();
        values.record(&mut Fields(&mut line));
        self.push(line);
    }
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn event(&self, event: &Event<'_>) {
        let mut line = event.metadata().level().to_string();
        event.record(&mut Fields(&mut line));
        self.push(line);
    }
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
}

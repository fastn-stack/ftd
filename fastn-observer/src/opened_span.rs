// borrowed from https://github.com/QnnOkabayashi/tracing-forest/ (license: MIT)

pub struct OpenedSpan {
    span: fastn_observer::Span,
    start: std::time::Instant,
    last_enter: std::time::Instant,
}

impl OpenedSpan {
    fn new(attrs: &tracing::span::Attributes) -> Self {
        let mut fields = fastn_observer::FieldSet::default();

        attrs.record(
            &mut |field: &tracing::field::Field, value: &dyn std::fmt::Debug| {
                let value = format!("{:?}", value);
                fields.push(fastn_observer::Field::new(field.name(), value));
            },
        );

        let shared = fastn_observer::Shared {
            level: *attrs.metadata().level(),
            fields,
            on: std::time::Duration::ZERO,
        };

        OpenedSpan {
            span: fastn_observer::Span::new(shared, attrs.metadata().name()),
            start: std::time::Instant::now(),
            last_enter: std::time::Instant::now(),
        }
    }

    fn enter(&mut self) {
        self.last_enter = std::time::Instant::now();
    }

    fn exit(&mut self) {
        self.span.total_duration += self.last_enter.elapsed();
    }

    fn close(self) -> fastn_observer::Span {
        self.span
    }

    fn record_event(&mut self, event: fastn_observer::Event) {
        self.span.nodes.push(fastn_observer::Tree::Event(event));
    }

    fn record_span(&mut self, span: fastn_observer::Span) {
        self.span.inner_duration += span.total_duration;
        self.span.nodes.push(fastn_observer::Tree::Span(span));
    }
}

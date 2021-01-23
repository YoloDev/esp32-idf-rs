use std::marker::PhantomData;

use anyhow::Result;
use fmt::{FormatEvent, FormatFields};
use termcolor::{ColorChoice, StandardStream};
use tracing::{metadata::LevelFilter, Event, Subscriber};
use tracing_subscriber::{fmt::MakeWriter, layer, registry::LookupSpan};

mod fmt;

struct Layer<S, N, E, W> {
  make_writer: W,
  fmt_event: E,
  _fmt_field: PhantomData<N>,
  _inner: PhantomData<S>,
}

impl<S, N, E, W> layer::Layer<S> for Layer<S, N, E, W>
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'writer> FormatFields<'writer> + 'static,
  E: FormatEvent<S, N> + 'static,
  W: MakeWriter + 'static,
  W::Writer: fmt::WriteColor,
{
  fn on_event(&self, event: &Event<'_>, ctx: layer::Context<'_, S>) {
    let mut writer = self.make_writer.make_writer();
    self
      .fmt_event
      .format_event(&ctx, &mut writer, event)
      .unwrap()
  }
}

impl<S, F, W> Layer<S, fmt::Pretty, fmt::Pretty, fmt::WriterFactory<F, W>>
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  F: Fn(ColorChoice) -> W + 'static,
  W: termcolor::WriteColor + 'static,
{
  fn new(color: ColorChoice, factory: F) -> Self {
    Self {
      make_writer: fmt::WriterFactory::new(color, factory),
      fmt_event: fmt::Pretty::default(),
      _fmt_field: PhantomData,
      _inner: PhantomData,
    }
  }
}

pub(crate) fn init(colors: ColorChoice, filter: LevelFilter) {
  use tracing_subscriber::prelude::*;

  let fmt_layer = Layer::new(colors, StandardStream::stdout);
  tracing_subscriber::registry()
    .with(filter)
    .with(fmt_layer)
    .init();

  // let writer_factory = WriterFactory(colors, StandardStream::stdout);
  // todo!()
}

use std::{
  fmt,
  io::{self, BufRead},
};

use termcolor::{Color, ColorChoice, ColorSpec};
use tracing::{field, Event, Level, Subscriber};
use tracing_log::NormalizeEvent;
use tracing_subscriber::{
  field::{MakeOutput, RecordFields, VisitFmt, VisitOutput},
  fmt::MakeWriter,
  layer,
  registry::LookupSpan,
};

pub trait WriteColor: fmt::Write + io::Write {
  /// Set the color settings of the writer.
  ///
  /// Subsequent writes to this writer will use these settings until either
  /// `reset` is called or new color settings are set.
  ///
  /// If there was a problem setting the color settings, then an error is
  /// returned.
  fn set_color(&mut self, spec: &ColorSpec) -> fmt::Result;

  /// Reset the current color settings to their original settings.
  ///
  /// If there was a problem resetting the color settings, then an error is
  /// returned.
  fn reset(&mut self) -> fmt::Result;

  fn as_writer(&mut self) -> &mut dyn fmt::Write;
}

pub trait WriteColorExt: WriteColor {
  fn with_color<F>(&mut self, color: &ColorSpec, write: F) -> fmt::Result
  where
    F: FnOnce(&mut Self) -> fmt::Result;

  fn write_color_str(&mut self, str: &str, color: &ColorSpec) -> fmt::Result;
}

impl<W: WriteColor + ?Sized> WriteColorExt for W {
  #[inline]
  fn with_color<F>(&mut self, color: &ColorSpec, write: F) -> fmt::Result
  where
    F: FnOnce(&mut Self) -> fmt::Result,
  {
    self.set_color(color)?;
    write(self)?;
    self.reset()?;

    Ok(())
  }

  #[inline]
  fn write_color_str(&mut self, str: &str, color: &ColorSpec) -> fmt::Result {
    self.with_color(color, |w| w.write_str(str))
  }
}

pub struct Writer<W: termcolor::WriteColor>(pub(crate) W);

impl<W: termcolor::WriteColor> io::Write for Writer<W> {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.0.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> io::Result<()> {
    self.0.flush()
  }

  #[inline]
  fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
    self.0.write_vectored(bufs)
  }

  #[inline]
  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    self.0.write_all(buf)
  }

  #[inline]
  fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
    self.0.write_fmt(fmt)
  }

  #[inline]
  fn by_ref(&mut self) -> &mut Self
  where
    Self: Sized,
  {
    self
  }
}

impl<W: termcolor::WriteColor> fmt::Write for Writer<W> {
  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    match self.0.write(s.as_ref()) {
      Ok(_) => Ok(()),
      Err(_) => Err(fmt::Error),
    }
  }
}

impl<W: termcolor::WriteColor> WriteColor for Writer<W> {
  #[inline]
  fn set_color(&mut self, spec: &ColorSpec) -> fmt::Result {
    match self.0.set_color(spec) {
      Ok(_) => Ok(()),
      Err(_) => Err(fmt::Error),
    }
  }

  #[inline]
  fn reset(&mut self) -> fmt::Result {
    match self.0.reset() {
      Ok(_) => Ok(()),
      Err(_) => Err(fmt::Error),
    }
  }

  #[inline]
  fn as_writer(&mut self) -> &mut dyn fmt::Write {
    self
  }
}

pub trait FormatEvent<S, N>
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'a> FormatFields<'a> + 'static,
{
  /// Write a log message for `Event` in `Context` to the given `Write`.
  fn format_event(
    &self,
    ctx: &layer::Context<'_, S>,
    writer: &mut dyn WriteColor,
    event: &Event<'_>,
  ) -> fmt::Result;
}

pub trait FormatFields<'writer> {
  /// Format the provided `fields` to the provided `writer`, returning a result.
  fn format_fields<R: RecordFields>(
    &self,
    writer: &'writer mut dyn WriteColor,
    fields: R,
  ) -> fmt::Result;

  // /// Record additional field(s) on an existing span.
  // ///
  // /// By default, this appends a space to the current set of fields if it is
  // /// non-empty, and then calls `self.format_fields`. If different behavior is
  // /// required, the default implementation of this method can be overridden.
  // fn add_fields(&self, current: &'writer mut String, fields: &span::Record<'_>) -> io::Result {
  //   if !current.is_empty() {
  //     current.push(' ');
  //   }
  //   self.format_fields(current, fields)
  // }
}

impl<'writer, M> FormatFields<'writer> for M
where
  M: MakeOutput<&'writer mut dyn WriteColor, io::Result<()>>,
  M::Visitor: VisitFmt + VisitOutput<io::Result<()>>,
{
  fn format_fields<R: RecordFields>(
    &self,
    writer: &'writer mut dyn WriteColor,
    fields: R,
  ) -> fmt::Result {
    let mut v = self.make_visitor(writer);
    fields.record(&mut v);
    v.finish()
  }
}

pub struct WriterFactory<F, W>(ColorChoice, F)
where
  F: Fn(ColorChoice) -> W,
  W: termcolor::WriteColor;

impl<F, W> WriterFactory<F, W>
where
  F: Fn(ColorChoice) -> W,
  W: termcolor::WriteColor,
{
  pub fn new(color: ColorChoice, factory: F) -> Self {
    Self(color, factory)
  }
}

impl<F, W> MakeWriter for WriterFactory<F, W>
where
  F: Fn(ColorChoice) -> W,
  W: termcolor::WriteColor,
{
  type Writer = Writer<W>;

  #[inline]
  fn make_writer(&self) -> Self::Writer {
    Writer(self.1(self.0))
  }
}

/// An excessively pretty, human-readable event formatter.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pretty {}

impl Default for Pretty {
  #[inline]
  fn default() -> Self {
    Pretty {}
  }
}

impl<'writer> FormatFields<'writer> for Pretty {
  fn format_fields<R: RecordFields>(
    &self,
    writer: &'writer mut dyn WriteColor,
    fields: R,
  ) -> fmt::Result {
    let mut v = PrettyVisitor::new(writer);
    fields.record(&mut v);
    v.finish()
  }
}

pub struct PrettyVisitor<'a> {
  writer: &'a mut dyn WriteColor,
  result: fmt::Result,
}

impl<'a> PrettyVisitor<'a> {
  pub fn new(writer: &'a mut dyn WriteColor) -> Self {
    Self {
      writer,
      result: Ok(()),
    }
  }
}

// TODO: Get creative with coloring
impl<'a> field::Visit for PrettyVisitor<'a> {
  fn record_debug(&mut self, field: &field::Field, value: &dyn fmt::Debug) {
    if self.result.is_err() {
      return;
    }

    let writer = self.writer.as_writer();
    self.result = match field.name() {
      "message" => write!(writer, "{:?}", value),
      // Skip fields that are actually log metadata that have already been handled
      name if name.starts_with("log.") => Ok(()),
      name if name.starts_with("r#") => write!(writer, "{}: {:?}", &name[2..], value),
      name => write!(writer, "{}: {:?}", name, value),
    };
  }

  fn record_i64(&mut self, field: &field::Field, value: i64) {
    self.record_debug(field, &value)
  }

  fn record_u64(&mut self, field: &field::Field, value: u64) {
    self.record_debug(field, &value)
  }

  fn record_bool(&mut self, field: &field::Field, value: bool) {
    self.record_debug(field, &value)
  }

  fn record_str(&mut self, field: &field::Field, value: &str) {
    if self.result.is_err() {
      return;
    }

    if field.name() == "message" {
      self.record_debug(field, &format_args!("{}", value))
    } else {
      self.record_debug(field, &value)
    }
  }

  fn record_error(&mut self, field: &field::Field, value: &(dyn std::error::Error + 'static)) {
    self.record_debug(field, &format_args!("{}", value))
  }
}
impl<'a> VisitOutput<fmt::Result> for PrettyVisitor<'a> {
  fn finish(self) -> fmt::Result {
    self.result
  }
}

trait ColorSpecExt {
  fn from_fg(color: Color) -> Self;
}

impl ColorSpecExt for ColorSpec {
  #[inline]
  fn from_fg(color: Color) -> Self {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color));
    spec
  }
}

impl Pretty {
  const SPACE: &'static str = " ";
  // const NEW_LINE: &'static str = "\n";
  // const ARROW: &'static str = "↳";
  // const DASH: &'static str = "-";
  // const DOT: &'static str = "•";
  // const FW: &'static str = "❯";
  const PIPE: &'static str = "│";

  const COLORS: &'static [Color] = &[
    Color::Rgb(0x00, 0x00, 0xCC),
    Color::Rgb(0x00, 0x00, 0xFF),
    Color::Rgb(0x00, 0x33, 0xCC),
    Color::Rgb(0x00, 0x33, 0xFF),
    Color::Rgb(0x00, 0x66, 0xCC),
    Color::Rgb(0x00, 0x66, 0xFF),
    Color::Rgb(0x00, 0x99, 0xCC),
    Color::Rgb(0x00, 0x99, 0xFF),
    Color::Rgb(0x00, 0xCC, 0x00),
    Color::Rgb(0x00, 0xCC, 0x33),
    Color::Rgb(0x00, 0xCC, 0x66),
    Color::Rgb(0x00, 0xCC, 0x99),
    Color::Rgb(0x00, 0xCC, 0xCC),
    Color::Rgb(0x00, 0xCC, 0xFF),
    Color::Rgb(0x33, 0x00, 0xCC),
    Color::Rgb(0x33, 0x00, 0xFF),
    Color::Rgb(0x33, 0x33, 0xCC),
    Color::Rgb(0x33, 0x33, 0xFF),
    Color::Rgb(0x33, 0x66, 0xCC),
    Color::Rgb(0x33, 0x66, 0xFF),
    Color::Rgb(0x33, 0x99, 0xCC),
    Color::Rgb(0x33, 0x99, 0xFF),
    Color::Rgb(0x33, 0xCC, 0x00),
    Color::Rgb(0x33, 0xCC, 0x33),
    Color::Rgb(0x33, 0xCC, 0x66),
    Color::Rgb(0x33, 0xCC, 0x99),
    Color::Rgb(0x33, 0xCC, 0xCC),
    Color::Rgb(0x33, 0xCC, 0xFF),
    Color::Rgb(0x66, 0x00, 0xCC),
    Color::Rgb(0x66, 0x00, 0xFF),
    Color::Rgb(0x66, 0x33, 0xCC),
    Color::Rgb(0x66, 0x33, 0xFF),
    Color::Rgb(0x66, 0xCC, 0x00),
    Color::Rgb(0x66, 0xCC, 0x33),
    Color::Rgb(0x99, 0x00, 0xCC),
    Color::Rgb(0x99, 0x00, 0xFF),
    Color::Rgb(0x99, 0x33, 0xCC),
    Color::Rgb(0x99, 0x33, 0xFF),
    Color::Rgb(0x99, 0xCC, 0x00),
    Color::Rgb(0x99, 0xCC, 0x33),
    Color::Rgb(0xCC, 0x00, 0x00),
    Color::Rgb(0xCC, 0x00, 0x33),
    Color::Rgb(0xCC, 0x00, 0x66),
    Color::Rgb(0xCC, 0x00, 0x99),
    Color::Rgb(0xCC, 0x00, 0xCC),
    Color::Rgb(0xCC, 0x00, 0xFF),
    Color::Rgb(0xCC, 0x33, 0x00),
    Color::Rgb(0xCC, 0x33, 0x33),
    Color::Rgb(0xCC, 0x33, 0x66),
    Color::Rgb(0xCC, 0x33, 0x99),
    Color::Rgb(0xCC, 0x33, 0xCC),
    Color::Rgb(0xCC, 0x33, 0xFF),
    Color::Rgb(0xCC, 0x66, 0x00),
    Color::Rgb(0xCC, 0x66, 0x33),
    Color::Rgb(0xCC, 0x99, 0x00),
    Color::Rgb(0xCC, 0x99, 0x33),
    Color::Rgb(0xCC, 0xCC, 0x00),
    Color::Rgb(0xCC, 0xCC, 0x33),
    Color::Rgb(0xFF, 0x00, 0x00),
    Color::Rgb(0xFF, 0x00, 0x33),
    Color::Rgb(0xFF, 0x00, 0x66),
    Color::Rgb(0xFF, 0x00, 0x99),
    Color::Rgb(0xFF, 0x00, 0xCC),
    Color::Rgb(0xFF, 0x00, 0xFF),
    Color::Rgb(0xFF, 0x33, 0x00),
    Color::Rgb(0xFF, 0x33, 0x33),
    Color::Rgb(0xFF, 0x33, 0x66),
    Color::Rgb(0xFF, 0x33, 0x99),
    Color::Rgb(0xFF, 0x33, 0xCC),
    Color::Rgb(0xFF, 0x33, 0xFF),
    Color::Rgb(0xFF, 0x66, 0x00),
    Color::Rgb(0xFF, 0x66, 0x33),
    Color::Rgb(0xFF, 0x99, 0x00),
    Color::Rgb(0xFF, 0x99, 0x33),
    Color::Rgb(0xFF, 0xCC, 0x00),
    Color::Rgb(0xFF, 0xCC, 0x33),
  ];

  fn heading(level: &Level) -> &'static str {
    match *level {
      Level::TRACE => "TRACE",
      Level::DEBUG => "DEBUG",
      Level::INFO => " INFO",
      Level::WARN => " WARN",
      Level::ERROR => "ERROR",
    }
  }

  fn prefix_color(level: &Level) -> ColorSpec {
    match *level {
      Level::TRACE => ColorSpec::from_fg(Color::Cyan),
      Level::DEBUG => ColorSpec::from_fg(Color::Blue),
      Level::INFO => ColorSpec::from_fg(Color::Green),
      Level::WARN => ColorSpec::from_fg(Color::Yellow),
      Level::ERROR => ColorSpec::from_fg(Color::Red),
    }
  }

  fn write_prefix(writer: &mut dyn WriteColor, level: &Level) -> fmt::Result {
    writer.with_color(&Self::prefix_color(level), |w| {
      w.write_str(Self::heading(level))?;
      w.write_str(Pretty::SPACE)?;
      w.write_str(Pretty::PIPE)?;
      w.write_str(Pretty::SPACE)?;
      Ok(())
    })
  }
}

impl<S, N> FormatEvent<S, N> for Pretty
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'a> FormatFields<'a> + 'static,
{
  fn format_event(
    &self,
    _ctx: &layer::Context<'_, S>,
    writer: &mut dyn WriteColor,
    event: &Event<'_>,
  ) -> fmt::Result {
    use fmt::Write;
    let normalized_meta = event.normalized_metadata();
    let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
    let target = meta.target();
    let level = meta.level();
    if target.starts_with("bindgen") {
      return Ok(());
    }

    let mut event_writer = PrefixWriter::new(writer, |writer| Pretty::write_prefix(writer, level));

    let target_hash = seahash::hash(target.as_ref());
    let target_color = Pretty::COLORS[target_hash as usize % Pretty::COLORS.len()];
    event_writer.write_color_str(target, &ColorSpec::from_fg(target_color))?;
    event_writer.write_str(Pretty::SPACE)?;
    let mut v = PrettyVisitor::new(&mut event_writer);
    event.record(&mut v);
    v.finish()?;
    writer.reset()?;
    writer.write_char('\n')?;

    Ok(())
  }
}

struct PrefixWriter<'a, F>
where
  F: for<'w> Fn(&'w mut dyn WriteColor) -> fmt::Result,
{
  writer: &'a mut dyn WriteColor,
  line_start: bool,
  write_prefix: F,
  color: ColorSpec,
}

impl<'a, F> PrefixWriter<'a, F>
where
  F: for<'w> Fn(&'w mut dyn WriteColor) -> fmt::Result,
{
  fn new(writer: &'a mut dyn WriteColor, write_prefix: F) -> Self {
    Self {
      writer,
      line_start: true,
      write_prefix,
      color: ColorSpec::new(),
    }
  }

  fn maybe_write_prefix(&mut self) -> io::Result<()> {
    if self.line_start {
      self.line_start = false;
      match (&self.write_prefix)(self.writer).and_then(|_| self.writer.set_color(&self.color)) {
        Ok(_) => Ok(()),
        Err(_) => Err(io::Error::new(
          io::ErrorKind::Other,
          "failed to write prefix",
        )),
      }?;
    }

    Ok(())
  }
}

impl<'a, F> io::Write for PrefixWriter<'a, F>
where
  F: for<'w> Fn(&'w mut dyn WriteColor) -> fmt::Result,
{
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let len = buf.len();
    let mut cursor = io::Cursor::new(buf);
    let mut buf = String::with_capacity(buf.len());
    while cursor.read_line(&mut buf)? > 0 {
      self.maybe_write_prefix()?;
      self.writer.write_all(buf.as_ref())?;
      if buf.ends_with('\n') {
        self.line_start = true;
      }
    }

    Ok(len)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.writer.flush()
  }
}
impl<'a, F> fmt::Write for PrefixWriter<'a, F>
where
  F: for<'w> Fn(&'w mut dyn WriteColor) -> fmt::Result,
{
  #[inline]
  fn write_str(&mut self, s: &str) -> fmt::Result {
    match <Self as io::Write>::write(self, s.as_ref()) {
      Ok(_) => Ok(()),
      Err(_) => Err(fmt::Error),
    }
  }
}

impl<'a, F> WriteColor for PrefixWriter<'a, F>
where
  F: for<'w> Fn(&'w mut dyn WriteColor) -> fmt::Result,
{
  #[inline]
  fn set_color(&mut self, spec: &ColorSpec) -> fmt::Result {
    debug_assert_eq!(spec.reset(), true, "color spec must have reset=true");
    self.color = spec.clone();
    self.writer.set_color(spec)
  }

  #[inline]
  fn reset(&mut self) -> fmt::Result {
    self.color = ColorSpec::new();
    self.writer.reset()
  }

  #[inline]
  fn as_writer(&mut self) -> &mut dyn fmt::Write {
    self
  }
}

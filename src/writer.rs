//! Contains high-level interface for an events-based XML emitter.
//!
//! The most important type in this module is `EventWriter` which allows writing an XML document
//! to some output stream.
extern crate alloc;

pub use self::config::EmitterConfig;
pub use self::emitter::EmitterError as Error;
pub use self::emitter::Result;
pub use self::events::XmlEvent;

use self::emitter::Emitter;

mod config;
mod emitter;
pub mod events;

/// A wrapper around an `std::io::Write` instance which emits XML document according to provided
/// events.
pub struct EventWriter {
    sink: alloc::string::String,
    emitter: Emitter,
}

impl EventWriter {
    /// Creates a new `EventWriter` out of an `std::io::Write` instance using the default
    /// configuration.
    #[inline]
    pub fn new() -> EventWriter {
        EventWriter::new_with_config(EmitterConfig::new())
    }

    /// Creates a new `EventWriter` out of an `std::io::Write` instance using the provided
    /// configuration.
    #[inline]
    pub fn new_with_config(config: EmitterConfig) -> EventWriter {
        EventWriter {
            sink: alloc::string::String::new(),
            emitter: Emitter::new(config),
        }
    }

    /// Writes the next piece of XML document according to the provided event.
    ///
    /// Note that output data may not exactly correspond to the written event because
    /// of various configuration options. For example, `XmlEvent::EndElement` may
    /// correspond to a separate closing element or it may cause writing an empty element.
    /// Another example is that `XmlEvent::CData` may be represented as characters in
    /// the output stream.
    pub fn write<'a, E>(&mut self, event: E) -> Result<()> where E: Into<XmlEvent<'a>> {
        match event.into() {
            XmlEvent::StartDocument { version, encoding, standalone } =>
                self.emitter.emit_start_document(&mut self.sink, version, encoding.unwrap_or("UTF-8"), standalone),
            XmlEvent::ProcessingInstruction { name, data } =>
                self.emitter.emit_processing_instruction(&mut self.sink, name, data),
            XmlEvent::StartElement { name, attributes, namespace } => {
                self.emitter.namespace_stack_mut().push_empty().checked_target().extend(namespace.as_ref());
                self.emitter.emit_start_element(&mut self.sink, name, &attributes)
            }
            XmlEvent::EndElement { name } => {
                let r = self.emitter.emit_end_element(&mut self.sink, name);
                self.emitter.namespace_stack_mut().try_pop();
                r
            }
            XmlEvent::Comment(content) => self.emitter.emit_comment(&mut self.sink, content),
            XmlEvent::CData(content) => Ok(self.emitter.emit_cdata(&mut self.sink, content)),
            XmlEvent::Characters(content) => Ok(self.emitter.emit_characters(&mut self.sink, content)),
        }
    }

    /// Returns a mutable reference to the underlying `Writer`.
    ///
    /// Note that having a reference to the underlying sink makes it very easy to emit invalid XML
    /// documents. Use this method with care. Valid use cases for this method include accessing
    /// methods like `Write::flush`, which do not emit new data but rather change the state
    /// of the stream itself.
    pub fn inner_mut(&mut self) -> &mut alloc::string::String {
        &mut self.sink
    }

    /// Unwraps this `EventWriter`, returning the underlying writer.
    ///
    /// Note that this is a destructive operation: unwrapping a writer and then wrapping
    /// it again with `EventWriter::new()` will create a fresh writer whose state will be
    /// blank; for example, accumulated namespaces will be reset.
    pub fn into_inner(self) -> alloc::string::String {
        self.sink
    }
}

use std::{io::Write, mem::take, sync::Arc};

use anyhow::Error;
use parking_lot::Mutex;
use swc_common::errors::{DiagnosticBuilder, Emitter, Handler, Level};

use crate::convert_ast::converter::{convert_string, node_types::TYPE_PARSE_ERROR};

#[derive(Clone, Default)]
struct Writer(Arc<Mutex<Vec<u8>>>);

impl Write for Writer {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let mut lock = self.0.lock();

    lock.extend_from_slice(buf);

    Ok(buf.len())
  }
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

pub struct ErrorEmitter {
  wr: Box<Writer>,
}

impl Emitter for ErrorEmitter {
  fn emit(&mut self, db: &DiagnosticBuilder<'_>) {
    if db.level == Level::Error {
      let mut buffer = Vec::new();
      let mut pos: u32 = 0;
      if let Some(span) = db.span.primary_span() {
        pos = span.lo.0 - 1;
      };
      let message = &db.message[0].0;
      buffer.extend_from_slice(&(pos as u32).to_ne_bytes());
      convert_string(&mut buffer, message);
      let _ = self.wr.write(&buffer);
    }
  }
}

pub fn try_with_handler<F, Ret>(code: &str, op: F) -> Result<Ret, Vec<u8>>
where
  F: FnOnce(&Handler) -> Result<Ret, Error>,
{
  let wr = Box::<Writer>::default();

  let emitter = ErrorEmitter { wr: wr.clone() };

  let handler = Handler::with_emitter(true, false, Box::new(emitter));

  let result = op(&handler);

  match result {
    Ok(r) => Ok(r),
    Err(_) => {
      let mut buffer = TYPE_PARSE_ERROR.to_vec();
      if handler.has_errors() {
        let mut lock = wr.0.lock();
        let mut error_buffer = take(&mut *lock);
        let pos = u32::from_ne_bytes(error_buffer[0..4].try_into().unwrap());
        let mut utf_16_pos: u32 = 0;
        for (utf_8_pos, char) in code.char_indices() {
          if (utf_8_pos as u32) == pos {
            break;
          }
          utf_16_pos += char.len_utf16() as u32;
        }
        error_buffer[0..4].copy_from_slice(&utf_16_pos.to_ne_bytes());
        buffer.extend_from_slice(&error_buffer);
      } else {
        panic!("Unexpected error in parse")
      }
      Err(buffer)
    }
  }
}

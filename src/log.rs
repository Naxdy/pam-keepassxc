use std::io::{self, Write};

use pamsm::{LogLvl, Pam, PamLibExt};
use tracing::{Dispatch, Level};
use tracing_subscriber::{filter, fmt::MakeWriter, layer::SubscriberExt};

/// Return a new [`Dispatch`] that writes to the PAM syslog.
///
/// # Safety
///
/// This subscriber must be unregistered before the [`Pam`] is dropped, and this function may not
/// be called while there exists an active PAM dispatch.
pub unsafe fn pam_dispatch(pam: &Pam) -> Dispatch {
    let pam: Pam = unsafe { std::mem::transmute_copy(pam) };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi_sanitization(false)
                .with_writer(PamWriterFactory { pam }),
        )
        .with(
            filter::EnvFilter::builder()
                .with_default_directive(filter::LevelFilter::INFO.into())
                .from_env()
                .expect("should always parse env"),
        )
        .into()
}

pub struct PamWriterFactory {
    pam: Pam,
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for PamWriterFactory {}
unsafe impl Sync for PamWriterFactory {}

pub struct PamWriter<'a> {
    pam: &'a Pam,
    log_level: Level,
}

impl Write for PamWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.pam
            .syslog(
                match self.log_level {
                    Level::TRACE | Level::DEBUG => LogLvl::DEBUG,
                    Level::WARN => LogLvl::WARNING,
                    Level::ERROR => LogLvl::ERR,
                    _ => LogLvl::INFO,
                },
                &String::from_utf8_lossy(buf),
            )
            .map_err(|e| io::Error::other(format!("PAM error: {e}")))?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for PamWriterFactory {
    type Writer = PamWriter<'writer>;

    fn make_writer(&'writer self) -> Self::Writer {
        PamWriter {
            pam: &self.pam,
            log_level: Level::INFO,
        }
    }

    fn make_writer_for(&'writer self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        PamWriter {
            pam: &self.pam,
            log_level: *meta.level(),
        }
    }
}

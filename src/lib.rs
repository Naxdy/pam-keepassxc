// originates in macro
#![allow(unsafe_op_in_unsafe_fn)]

mod dbus;
mod log;

use eyre::Context;
use pamsm::{PamError, PamLibExt, PamServiceModule, pam_module};
use tracing::{error, info};
use tracing_subscriber::util::SubscriberInitExt;

use crate::{dbus::KeePassXcInterface, log::pam_dispatch};

struct PamKeepassXc;

impl PamServiceModule for PamKeepassXc {
    fn authenticate(pam: pamsm::Pam, _: pamsm::PamFlags, args: Vec<String>) -> pamsm::PamError {
        let _guard = unsafe { pam_dispatch(&pam) }.set_default();

        {
            let password = match pam.get_authtok(None) {
                Ok(Some(e)) => e.to_str().expect("should always parse password"),
                Ok(None) => return PamError::USER_UNKNOWN,
                Err(e) => {
                    error!("failed to get auth token with PAM error: {e}");

                    return e;
                }
            };

            if let Err(e) = try_unlock_databases(&args, password) {
                error!("error while trying to unlock databases: {e:?}");

                return PamError::SERVICE_ERR;
            }
        }

        PamError::SUCCESS
    }

    fn close_session(pam: pamsm::Pam, _: pamsm::PamFlags, _: Vec<String>) -> pamsm::PamError {
        let _guard = unsafe { pam_dispatch(&pam) }.set_default();

        let interface = match KeePassXcInterface::new() {
            Ok(e) => e,
            Err(e) => {
                error!("failed to initialize KeePassXC interface: {e}");

                return PamError::SERVICE_ERR;
            }
        };

        if let Err(e) = interface.lock_all_databases() {
            error!("failed to lock databases: {e:?}");
            PamError::SERVICE_ERR
        } else {
            info!("dispatched lock databases request");

            PamError::SUCCESS
        }
    }

    fn open_session(
        pam: pamsm::Pam,
        pam_flags: pamsm::PamFlags,
        args: Vec<String>,
    ) -> pamsm::PamError {
        Self::authenticate(pam, pam_flags, args)
    }

    fn chauthtok(
        pam: pamsm::Pam,
        pam_flags: pamsm::PamFlags,
        args: Vec<String>,
    ) -> pamsm::PamError {
        Self::authenticate(pam, pam_flags, args)
    }

    fn setcred(_: pamsm::Pam, _: pamsm::PamFlags, _: Vec<String>) -> pamsm::PamError {
        PamError::IGNORE
    }

    fn acct_mgmt(_: pamsm::Pam, _: pamsm::PamFlags, _: Vec<String>) -> pamsm::PamError {
        PamError::IGNORE
    }
}

fn try_unlock_databases<'a>(
    paths: impl IntoIterator<Item = &'a String>,
    password: &'a str,
) -> eyre::Result<()> {
    let interface = KeePassXcInterface::new().wrap_err("failed to initialize dbus interface")?;

    for db in paths {
        interface
            .unlock_database(db, password)
            .wrap_err_with(|| format!("failed to unlock database {db}"))?;

        info!("dispatched unlock request for {db}");
    }

    Ok(())
}

pam_module!(PamKeepassXc);

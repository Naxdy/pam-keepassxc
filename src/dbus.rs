use zbus::{blocking::Connection, proxy};

#[proxy(
    interface = "org.keepassxc.KeePassXC.MainWindow",
    default_service = "org.keepassxc.KeePassXC.MainWindow",
    default_path = "/keepassxc"
)]
trait KeepassXc {
    #[zbus(name = "openDatabase")]
    async fn open_database(&self, file_name: &str, pw: &str) -> zbus::Result<()>;

    #[zbus(name = "openDatabase")]
    async fn open_database_with_keyfile(
        &self,
        file_name: &str,
        pw: &str,
        key_file: &str,
    ) -> zbus::Result<()>;

    #[zbus(name = "lockAllDatabases")]
    async fn lock_all_databases(&self) -> zbus::Result<()>;
}

pub struct KeePassXcInterface<'a> {
    proxy: KeepassXcProxyBlocking<'a>,
}

impl KeePassXcInterface<'_> {
    pub fn new() -> zbus::Result<Self> {
        let conn = Connection::session()?;

        let proxy = KeepassXcProxyBlocking::new(&conn)?;

        Ok(Self { proxy })
    }

    pub fn unlock_database(&self, path: &str, password: &str) -> zbus::Result<()> {
        self.proxy.open_database(path, password)
    }

    pub fn lock_all_databases(&self) -> zbus::Result<()> {
        self.proxy.lock_all_databases()
    }
}

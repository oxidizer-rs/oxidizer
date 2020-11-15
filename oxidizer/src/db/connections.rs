use async_trait::async_trait;
use mobc::Manager;
use tokio_postgres::{Client, Config, NoTls};

use crate::Error;

pub(crate) struct ConnectionManager {
    pub provider: Box<dyn ConnectionProvider>,
}

#[async_trait]
impl Manager for ConnectionManager {
    type Connection = Client;
    type Error = tokio_postgres::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        self.provider.connect().await
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        conn.simple_query("").await?;
        Ok(conn)
    }
}

#[async_trait]
pub trait ConnectionProvider: Send + Sync + 'static {
    async fn connect(&self) -> Result<Client, tokio_postgres::Error>;
}

pub async fn create_connection_provider(
    config: Config,
    ca_file: Option<&str>,
) -> Result<Box<dyn ConnectionProvider>, Error> {
    let prov: Box<dyn ConnectionProvider> = if let Some(ca_file) = ca_file {
        cfg_if::cfg_if! {
            if #[cfg(feature = "tls-rustls")] {
                tls_rustls::create_rustls_provider(config, ca_file).await?
            } else if #[cfg(feature = "tls-openssl")] {
                tls_openssl::create_openssl_provider(config, ca_file).await?
            } else {
                eprintln!("[WARN] no TLS provider found, reverting to unencrypted connection");
                no_tls(config)
            }
        }
    } else {
        no_tls(config)
    };

    Ok(prov)
}

struct NoTlsConnectionProvider {
    config: Config,
}

#[async_trait]
impl ConnectionProvider for NoTlsConnectionProvider {
    async fn connect(&self) -> Result<Client, tokio_postgres::Error> {
        let (client, conn) = self.config.connect(NoTls).await?;
        mobc::spawn(conn);
        Ok(client)
    }
}

fn no_tls(config: Config) -> Box<dyn ConnectionProvider> {
    Box::new(NoTlsConnectionProvider { config })
}

#[cfg(feature = "tls-openssl")]
mod tls_openssl {
    use super::*;

    use openssl::ssl::{SslConnector, SslMethod};

    use postgres_openssl::MakeTlsConnector;
    use tokio_postgres::{Client, Config};

    struct OpensslConnectionProvider {
        config: Config,
        tls: MakeTlsConnector,
    }

    #[async_trait]
    impl ConnectionProvider for OpensslConnectionProvider {
        async fn connect(&self) -> Result<Client, tokio_postgres::Error> {
            let (client, conn) = self.config.connect(self.tls.clone()).await?;
            mobc::spawn(conn);
            Ok(client)
        }
    }

    fn sync_build_ssl_connector(ca_file: String) -> Result<SslConnector, Error> {
        let mut builder = SslConnector::builder(SslMethod::tls()).map_err(Error::OpensslError)?;

        builder.set_ca_file(&ca_file).map_err(Error::OpensslError)?;

        Ok(builder.build())
    }

    pub async fn create_openssl_provider(
        config: Config,
        ca_file: &str,
    ) -> Result<Box<dyn ConnectionProvider>, Error> {
        let file = ca_file.to_string();
        let connector =
            tokio::task::spawn_blocking(move || sync_build_ssl_connector(file)).await??;

        Ok(Box::new(OpensslConnectionProvider {
            config,
            tls: MakeTlsConnector::new(connector),
        }))
    }
}

#[cfg(feature = "tls-rustls")]
mod tls_rustls {
    use super::*;

    use std::fs::File;
    use std::io::BufReader;

    use rustls::{ClientConfig, RootCertStore};
    use tokio_postgres_rustls::MakeRustlsConnect;

    struct RustlsConnectionProvider {
        config: Config,
        tls: MakeRustlsConnect,
    }

    #[async_trait]
    impl ConnectionProvider for RustlsConnectionProvider {
        async fn connect(&self) -> Result<Client, tokio_postgres::Error> {
            let (client, conn) = self.config.connect(self.tls.clone()).await?;
            mobc::spawn(conn);
            Ok(client)
        }
    }

    fn sync_initialise_root_store(ca_file: String) -> Result<RootCertStore, Error> {
        let file = File::open(&ca_file).map_err(|err| Error::Other(err.to_string()))?;
        let mut reader = BufReader::new(file);

        let mut root_store = RootCertStore::empty();
        root_store
            .add_pem_file(&mut reader)
            .map_err(|_| Error::RustlsError("Failed to read certificate file".to_string()))?;

        Ok(root_store)
    }

    pub async fn create_rustls_provider(
        config: Config,
        ca_file: &str,
    ) -> Result<Box<dyn ConnectionProvider>, Error> {
        let mut tls_conf = ClientConfig::new();

        let file = ca_file.to_string();
        tls_conf.root_store =
            tokio::task::spawn_blocking(move || sync_initialise_root_store(file)).await??;

        Ok(Box::new(RustlsConnectionProvider {
            config,
            tls: MakeRustlsConnect::new(tls_conf),
        }))
    }
}

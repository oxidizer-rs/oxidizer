use async_trait::async_trait;
use tokio_postgres::{Client, Config, NoTls};

use crate::Error;

#[async_trait]
pub trait ConnectionProvider: Send + Sync + 'static {
    async fn connect(&self) -> Result<Client, tokio_postgres::Error>;
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

fn no_tls(config: Config) -> NoTlsConnectionProvider {
    NoTlsConnectionProvider { config }
}

pub fn create_connection_provider(
    config: Config,
    ca_file: Option<&str>,
) -> Result<Box<dyn ConnectionProvider>, Error> {
    #[allow(clippy::redundant_clone)]
    let prov: Box<dyn ConnectionProvider> = Box::new(no_tls(config.clone()));

    #[allow(unused_variables)]
    if let Some(ca_file) = ca_file {
        #[cfg(target_feature = "tls-openssl")]
        {
            prov = tls_openssl::new(config.clone(), ca_file)?;
        }
        #[cfg(target_feature = "tls-rustls")]
        {
            prov = tls_rustls::new(config.clone(), ca_file)?;
        }
    }

    Ok(prov)
}

cfg_if::cfg_if! {
if #[cfg(target_feature = "tls-openssl")] {

pub(crate) mod tls_openssl {
    use super::*;

    use openssl::ssl::{SslConnector, SslMethod};
    use postgres_openssl::MakeTlsConnector;
    use tokio_postgres::{Client, Config};

    pub(crate) struct OpensslConnectionProvider {
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

    pub fn new(config: Config, ca_file: &str) -> Result<Box<dyn ConnectionProvider>, Error> {
        let mut builder = SslConnector::builder(SslMethod::tls()).map_err(Error::OpensslError)?;

        builder.set_ca_file(ca_file).map_err(Error::OpensslError)?;

        Ok(Box::new(OpensslConnectionProvider {
            config,
            tls: MakeTlsConnector::new(builder.build()),
        }))
    }
}
}}

cfg_if::cfg_if! {
if #[cfg(target_feature = "tls-rustls")] {

pub(crate) mod tls_rustls {
    use super::*;

    use rustls;
    use tokio_postgres_rustls::MakeRustlsConnect;

    pub(crate) struct RustlsConnectionProvider {
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

    pub fn new(config: Config, ca_file: &str) -> Result<Box<dyn ConnectionProvider>, Error> {
        let mut tls_conf = rustls::ClientConfig::new();

        let file = std::fs::File::open(ca_file).map_err(|err| Error::Other(err.to_string()))?;
        let reader = BufReader::new(file);

        let mut root_store = rustls::RootCertStore::empty();
        root_store
            .add_pem_file(reader)
            .map_err(|| Error::RustlsError("Failed to read certificate file".to_string()))?;

        tls_conf.root_store = root_store;

        Ok(Box::new(RustlsConnectionProvider {
            config,
            tls: MakeRustlsConnect::new(tls_conf),
        }))
    }
}
}}

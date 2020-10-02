use async_trait::async_trait;
use tokio_postgres::{Client, Config, NoTls};

use crate::Error;

#[async_trait]
pub trait ConnectionProvider: Send + Sync + 'static {
    async fn connect(&self) -> Result<Client, tokio_postgres::Error>;
}

pub fn create_connection_provider(
    config: Config,
    ca_file: Option<&str>,
) -> Result<Box<dyn ConnectionProvider>, Error> {
    let prov: Box<dyn ConnectionProvider> = if let Some(ca_file) = ca_file {
        new_tls(config, ca_file)?
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

cfg_if::cfg_if! {
    if #[cfg(not(any(feature = "tls-openssl", feature = "tls-rustls")))] {
        fn new_tls(config: Config, ca_file: &str) -> Result<Box<dyn ConnectionProvider>, Error> {
            eprintln!("[WARN] no TLS provider configured");
            Ok(no_tls(config))
        }
    }
}

cfg_if::cfg_if! {
if #[cfg(feature = "tls-openssl")] {

use tls_openssl::new_tls;

pub(crate) mod tls_openssl {
    use super::*;

    use openssl::ssl::{SslConnector, SslMethod, SslOptions};

    use tokio_postgres::{Client, Config};
    use postgres_openssl::MakeTlsConnector;

    struct OpensslConnectionProvider {
        pub(crate) config: Config,
        pub(crate) tls: MakeTlsConnector,
    }

    #[async_trait]
    impl ConnectionProvider for OpensslConnectionProvider {
        async fn connect(&self) -> Result<Client, tokio_postgres::Error> {
            let (client, conn) = self.config.connect(self.tls.clone()).await?;
            mobc::spawn(conn);
            Ok(client)
        }
    }

    pub fn new_tls(config: Config, ca_file: &str) -> Result<Box<dyn ConnectionProvider>, Error> {
        let mut builder = SslConnector::builder(SslMethod::tls())
            .map_err(Error::OpensslError)?;

        builder.set_options(SslOptions::NO_TLSV1_3);
        builder.set_ca_file(ca_file).map_err(Error::OpensslError)?;

        Ok(Box::new(OpensslConnectionProvider {
            config,
            tls: MakeTlsConnector::new(builder.build()),
        }))
    }
}
}}

cfg_if::cfg_if! {
if #[cfg(feature = "tls-rustls")] {

use tls_rustls::new_tls;

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

    pub fn new_tls(config: Config, ca_file: &str) -> Result<Box<dyn ConnectionProvider>, Error> {
        let mut tls_conf = ClientConfig::new();

        let file = File::open(ca_file).map_err(|err| Error::Other(err.to_string()))?;
        let mut reader = BufReader::new(file);

        let mut root_store = RootCertStore::empty();
        root_store
            .add_pem_file(&mut reader)
            .map_err(|_| Error::RustlsError("Failed to read certificate file".to_string()))?;

        for root in &root_store.roots {
            println!("{:?}", root);
        }

        tls_conf.root_store = root_store;

        Ok(Box::new(RustlsConnectionProvider {
            config,
            tls: MakeRustlsConnect::new(tls_conf),
        }))
    }
}
}}

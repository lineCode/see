use crate::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio_rustls::rustls::internal::pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use tokio_rustls::rustls::sign::{any_supported_type, CertifiedKey};
use tokio_rustls::rustls::{
    Certificate, NoClientAuth, PrivateKey, ResolvesServerCertUsingSNI, ServerConfig,
};
use tokio_rustls::TlsAcceptor;

fn load_certs<P: AsRef<Path>>(path: P) -> Vec<Certificate> {
    let file = File::open(&path)
        .unwrap_or_else(|err| exit!("Open '{}' failed\n{:?}", path.as_ref().display(), err));

    certs(&mut BufReader::new(file))
        .unwrap_or_else(|_| exit!("Load certs failed: {}", path.as_ref().display()))
}

fn load_keys<P: AsRef<Path>>(path: P) -> Vec<PrivateKey> {
    let p = path.as_ref().display();
    let file = || File::open(&path).unwrap_or_else(|err| exit!("Open '{}' failed\n{:?}", p, err));

    let keys = rsa_private_keys(&mut BufReader::new(file()))
        .unwrap_or_else(|_| exit!("Load rsa_private_keys failed: {}", &p));
    if !keys.is_empty() {
        return keys;
    }

    let keys = pkcs8_private_keys(&mut BufReader::new(file()))
        .unwrap_or_else(|_| exit!("Load pkcs8_private_keys failed: {}", &p));
    if !keys.is_empty() {
        return keys;
    }

    exit!("Load keys failed: '{}'", p)
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TLSContent {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub sni: String,
}

pub fn create_sni_server_config(group: Vec<TLSContent>) -> TlsAcceptor {
    let mut config = ServerConfig::new(NoClientAuth::new());
    let mut sni = ResolvesServerCertUsingSNI::new();

    for content in group {
        let certs = load_certs(content.cert);
        let mut keys = load_keys(content.key);
        let sign = any_supported_type(&keys.remove(0)).unwrap();
        let cert = CertifiedKey::new(certs, Arc::new(sign));

        sni.add(&content.sni, cert).unwrap();
    }

    config.cert_resolver = Arc::new(sni);
    config.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);

    TlsAcceptor::from(Arc::new(config))
}

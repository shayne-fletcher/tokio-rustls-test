use anyhow::Result;
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use rustls::{ClientConfig, ServerConfig};
use std::io::BufReader;
use std::sync::Arc;
use tokio_rustls::{TlsAcceptor, TlsConnector};

pub fn generate_self_signed_cert() -> Result<(String, String)> {
    let mut params = CertificateParams::new(vec!["localhost".to_string()]);
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "localhost");

    let cert = Certificate::from_params(params)?;
    let cert_pem = cert.serialize_pem()?;
    let key_pem = cert.serialize_private_key_pem();

    Ok((cert_pem, key_pem))
}

pub fn create_server_config(cert_pem: &str, key_pem: &str) -> Result<TlsAcceptor> {
    let cert_chain = rustls_pemfile::certs(&mut BufReader::new(cert_pem.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|v| rustls::Certificate(v.to_vec()))
        .collect();

    let mut private_keys = rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(key_pem.as_bytes()))
        .collect::<Result<Vec<_>, _>>()?;
    
    let private_key = private_keys
        .pop()
        .map(|k| rustls::PrivateKey(k.secret_pkcs8_der().to_vec()))
        .ok_or_else(|| anyhow::anyhow!("No private key found"))?;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

pub fn create_client_config() -> Result<TlsConnector> {
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(NoVerification))
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}

#[derive(Debug)]
struct NoVerification;

impl rustls::client::ServerCertVerifier for NoVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
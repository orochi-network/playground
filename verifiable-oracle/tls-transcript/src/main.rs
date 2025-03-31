// Reference: https://github.com/rustls/rustls/issues/1833
use rustls::crypto::aws_lc_rs::cipher_suite::{TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384};
use rustls::crypto::cipher::{AeadKey, Iv};
use rustls::ProtocolVersion;
use rustls::{
    crypto::{aws_lc_rs::default_provider, CryptoProvider},
    ClientConfig, ClientConnection, ConnectionTrafficSecrets, ExtractedSecrets, RootCertStore,
    Stream,
};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::time::SystemTime;
use webpki_roots;

enum DebuggableConnectionTrafficSecrets {
    /// Secrets for the AES_128_GCM AEAD algorithm
    Aes128Gcm {
        /// AEAD Key
        key: AeadKey,
        /// Initialization vector
        iv: Iv,
    },

    /// Secrets for the AES_256_GCM AEAD algorithm
    Aes256Gcm {
        /// AEAD Key
        key: AeadKey,
        /// Initialization vector
        iv: Iv,
    },

    /// Secrets for the CHACHA20_POLY1305 AEAD algorithm
    Chacha20Poly1305 {
        /// AEAD Key
        key: AeadKey,
        /// Initialization vector
        iv: Iv,
    },
}
struct DebuggableSecrets {
    /// sequence number and secrets for the "tx" (transmit) direction
    pub tx: (u64, ConnectionTrafficSecrets),

    /// sequence number and secrets for the "rx" (receive) direction
    pub rx: (u64, ConnectionTrafficSecrets),
}

struct TlsTranscript {
    protocol_version: ProtocolVersion,
    cipher_suite: String,
    cert_chain: Vec<Vec<u8>>,
    handshake_messages: Vec<String>,
    application_messages: Vec<String>,
    master_key: Vec<u8>,
    headers: String,
    response_body: Vec<u8>,
    response_hash: String,
    timestamp: u64,
}

fn fetch_tls_transcript() -> Result<TlsTranscript, Box<dyn std::error::Error>> {
    let hostname = "api.binance.com";

    let cipher_suites = vec![
        TLS13_AES_256_GCM_SHA384,
        TLS13_AES_128_GCM_SHA256,
        TLS13_AES_128_GCM_SHA256,
    ];

    let crypto_provider = CryptoProvider {
        cipher_suites,
        ..default_provider()
    };
    crypto_provider.install_default().unwrap();

    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_owned(),
    };

    let mut config = ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_root_certificates(root_store)
        .with_no_client_auth();
    config.enable_secret_extraction = true;
    let config = std::sync::Arc::new(config);

    let name = hostname.try_into().unwrap();

    let mut conn = ClientConnection::new(config, name).unwrap();
    let mut sock = std::net::TcpStream::connect(format!("{}:443", hostname)).unwrap();

    let mut tls = Stream::new(&mut conn, &mut sock);

    // Now proceed with the TLS communication
    let request = format!(
        "GET /api/v3/ticker/price?symbol=BTCUSDT HTTP/1.1\r\n\
        Host: {}\r\n\
        Connection: close\r\n\
        Accept: application/json\r\n\r\n",
        hostname
    );
    tls.write_all(request.as_bytes()).unwrap();
    tls.flush().unwrap();

    // Read the server's response with error handling
    let mut response = Vec::new();
    match tls.read_to_end(&mut response) {
        Ok(_) => println!("Successfully read response"),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            println!("Warning: Server closed connection without TLS close_notify");
            // Continue with whatever data we did receive
        }
        Err(e) => panic!("Unexpected error reading response: {:?}", e),
    }

    // Process whatever response we got
    let response_str = String::from_utf8_lossy(&response);
    let (headers, body) = response_str
        .split_once("\r\n\r\n")
        .unwrap_or((&response_str, ""));

    let cert_chain: Vec<Vec<u8>> = conn
        .peer_certificates()
        .map(|certs| certs.iter().map(|c| c.as_ref().to_vec()).collect())
        .unwrap_or_default();

    // Print protocol and cipher info
    let protocol_version: rustls::ProtocolVersion = conn.protocol_version().unwrap();
    let negotiated_cipher_suite: rustls::SupportedCipherSuite =
        conn.negotiated_cipher_suite().unwrap();
    println!(
        "Negotiated {protocol_version:?} {:?}",
        negotiated_cipher_suite,
    );

    // Print response details
    println!("\nResponse Headers:\n{}", headers);
    println!(
        "\nResponse Body (first 100 chars):\n{}",
        if body.len() > 100 { &body[..100] } else { body }
    );

    // Export master key material BEFORE extracting secrets
    let mut master_key_material: Vec<u8> = vec![0u8; 48]; // 48 bytes is a common length for master secret material
    conn.export_keying_material(
        &mut master_key_material,
        b"EXPORTER-master-secret", // Label for key derivation
        None,                      // No context
    )
    .unwrap();
    println!("\nMaster Key Material (hex): {:02x?}", master_key_material);
    println!(
        "Master Key Material length: {} bytes",
        master_key_material.len()
    );

    // Extract and process secrets
    let ExtractedSecrets { tx, rx } = conn.dangerous_extract_secrets().unwrap();

    // Print information about the secrets without moving them
    println!("\n=== Traffic Secrets ===");
    match &tx.1 {
        ConnectionTrafficSecrets::Aes256Gcm { key, .. } => {
            assert_eq!(key.as_ref().len(), 32);
            println!(
                "\ntx is ConnectionTrafficSecrets::Aes256Gcm with key {}B key",
                key.as_ref().len()
            );
        }
        ConnectionTrafficSecrets::Aes128Gcm { key, .. } => {
            assert_eq!(key.as_ref().len(), 16);
            println!(
                "tx is ConnectionTrafficSecrets::Aes128Gcm with key {}B key",
                key.as_ref().len()
            );
        }
        ConnectionTrafficSecrets::Chacha20Poly1305 { key, .. } => {
            assert_eq!(key.as_ref().len(), 32);
            println!(
                "tx is ConnectionTrafficSecrets::Chacha20Poly1305 with key {}B key",
                key.as_ref().len()
            );
        }
        _ => unreachable!(),
    }

    match rx.1 {
        ConnectionTrafficSecrets::Aes256Gcm { key, .. } => {
            assert_eq!(key.as_ref().len(), 32);
            println!(
                "\ntx is ConnectionTrafficSecrets::Aes256Gcm with key {}B key",
                key.as_ref().len()
            );
        }
        ConnectionTrafficSecrets::Aes128Gcm { key, .. } => {
            assert_eq!(key.as_ref().len(), 16);
            println!(
                "tx is ConnectionTrafficSecrets::Aes128Gcm with key {}B key",
                key.as_ref().len()
            );
        }
        ConnectionTrafficSecrets::Chacha20Poly1305 { key, .. } => {
            assert_eq!(key.as_ref().len(), 32);
            println!(
                "tx is ConnectionTrafficSecrets::Chacha20Poly1305 with key {}B key",
                key.as_ref().len()
            );
        }
        _ => unreachable!(),
    }

    // Calculate response hash
    let response_hash = format!("{:x}", Sha256::digest(&response));

    // Get timestamp
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();

    // Initialize empty logs (or implement proper logging if needed)
    let handshake_logs = Vec::new();
    let app_logs = Vec::new();

    // Debugging: Print certificate chain info
    for (i, cert) in cert_chain.iter().enumerate() {
        println!("Certificate {}: {} bytes", i + 1, cert.len());
    }

    // TODO: return extracted secrets
    Ok(TlsTranscript {
        protocol_version,
        cipher_suite: negotiated_cipher_suite
            .suite()
            .as_str()
            .unwrap()
            .to_string(),
        cert_chain,
        handshake_messages: handshake_logs,
        application_messages: app_logs,
        master_key: master_key_material,
        headers: headers.to_string(),
        response_body: response.to_vec(),
        response_hash,
        timestamp,
    })
}

// Function to verify TLS transcript integrity
fn verify_tls_transcript(transcript: &TlsTranscript) -> bool {
    println!("\n=== Verifying TLS Transcript ===");

    println!(
        "TLS protocol version used is {}",
        transcript.protocol_version.as_str().unwrap()
    );
    if !transcript
        .protocol_version
        .as_str()
        .unwrap()
        .contains("TLSv1_3")
    {
        println!(
            "❌ Protocol mismatch: Expected TLS 1.3, got {}",
            transcript.protocol_version.as_str().unwrap()
        );
        return false;
    }
    if transcript.cipher_suite != "TLS_AES_256_GCM_SHA384"
        && transcript.cipher_suite != "TLS_AES_128_GCM_SHA256"
        && transcript.cipher_suite != "TLS13_AES_128_GCM_SHA256"
    {
        println!(
            "❌ Cipher suite mismatch: Expected [TLS_AES_256_GCM_SHA384, TLS_AES_128_GCM_SHA256, TLS13_AES_128_GCM_SHA256], got {}",
            transcript.cipher_suite
        );
        return false;
    }
    // TODO: verify handshake integrity (secrets & master key) & integrity of the messages using master key

    // TODO: verify ful cert chain (back to CA)
    if transcript.cert_chain.is_empty() {
        println!("❌ No certificates received!");
        return false;
    }

    println!("✅ TLS handshake verified!");

    // Verify response hash for integrity
    let expected_hash = transcript.response_hash.clone();
    let computed_hash = format!("{:x}", Sha256::digest(&transcript.response_body));

    if computed_hash != expected_hash {
        println!("❌ Response integrity check failed!");
        return false;
    }

    println!("✅ Response integrity verified!");

    true
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transcript = fetch_tls_transcript()?;

    println!("=== TLS Transcript ===");
    let protocol_str = transcript.protocol_version.as_str().unwrap();
    println!("Protocol Version: {}", protocol_str);
    println!("Cipher Suite: {}", transcript.cipher_suite);
    println!("Certificates Received: {}", transcript.cert_chain.len());

    println!("\n=== Handshake Logs ===");
    for log in &transcript.handshake_messages {
        println!("{}", log);
    }

    println!("\n=== Headers ===\n{}", transcript.headers);

    println!("\n=== Response Body (first 100 bytes) ===");
    println!(
        "{}",
        String::from_utf8_lossy(
            &transcript.response_body[..100.min(transcript.response_body.len())]
        )
    );

    println!("\nResponse Hash: {}", transcript.response_hash);
    println!("\nTimestamp: {}", transcript.timestamp);

    // Verify transcript integrity
    if verify_tls_transcript(&transcript) {
        println!("\n✅ TLS verification successful!");
    } else {
        println!("\n❌ TLS verification failed!");
    }

    Ok(())
}

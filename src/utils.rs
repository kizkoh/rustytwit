use crypto::digest::Digest;
use crypto::sha2::Sha256;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::prelude::*;

#[derive(Debug)]
pub enum UtilsError {
    Io(io::Error),
    String(String),
}

impl From<io::Error> for UtilsError {
    fn from(err: io::Error) -> UtilsError {
        UtilsError::Io(err)
    }
}

impl From<String> for UtilsError {
    fn from(err: String) -> UtilsError {
        UtilsError::String(err)
    }
}

pub fn get_profile_image(profile_image_url: &String) -> Result<String, UtilsError> {
    let home_dir = match env::home_dir() {
        Some(home_dir) => home_dir,
        None => {
            error!("home directory is not set");
            panic!("home directory is not set")
        },
    };

    let cache_dir = home_dir
        .clone()
        .join(::vars::CACHE_DIR)
        .join("rustytwit")
        .join("images");

    let mut sha256 = Sha256::new();
    sha256.input_str(&profile_image_url);

    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    match File::open(cache_dir.join(sha256.result_str())) {
        Ok(_) => (),
        Err(_) => {
            match client.get(profile_image_url).send() {
                Ok(mut resp) => {
                    let mut body = vec![];
                    resp.read_to_end(&mut body).unwrap();
                    try!(
                        File::create(cache_dir.join(sha256.result_str()))?
                            .write_all(&body)
                    );
                },
                Err(err) => {
                    return Err(UtilsError::String(format!("{:?}", err)));
                },
            };
        },
    }

    let cache_path = cache_dir.join(sha256.result_str());
    return Ok(cache_path.to_str().unwrap().to_string());
}

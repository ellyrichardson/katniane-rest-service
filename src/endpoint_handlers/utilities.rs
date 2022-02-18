use std::{env, str};
use std::fs::File;
use std::io::Read;
use toml::Value;
use serde::Deserialize;
use std::string::String;
use openssl::rsa::{Rsa, Padding};
use openssl::symm::Cipher;

// REST Service Config
#[derive(Deserialize)]
pub struct Config {
    pub katniane_chain_address: String,
    pub katniane_chain_port: String,
    pub public_key_location: String,
    pub private_key_location: String,
}

pub fn load_service_config() -> Config {
    // Working directory of the application
    let mut env_dir: String = env::current_dir().unwrap().into_os_string().into_string().unwrap();
  
    // Prepare the whole environment directory of the Config.toml
    let config_file: &str = "/Config.toml";
    println!("Working directory is {:?}", &env_dir);
    env_dir.push_str(config_file);

    let cfstr = read_file_content_from_dir(&env_dir);
  
    println!("Config File: {}", &env_dir);
    println!("Config contents: {}", &cfstr);
  
    toml::from_str(&cfstr).unwrap()
  }

pub fn get_private_key() -> String {
    //let key_loc = read_file_content_from_dir(&load_service_config().private_key_location);

    // Read and return the key from the file provided in the key location
    // read_file_content_from_dir(&key_loc)
    read_file_content_from_dir(&load_service_config().private_key_location)
}

pub fn get_public_key() -> String {
    //let key_loc = read_file_content_from_dir(&load_service_config().public_key_location);

    // Read and return the key from the file provided in the key location
    //read_file_content_from_dir(&key_loc)
    read_file_content_from_dir(&load_service_config().public_key_location)
}

pub fn read_file_content_from_dir(file_dir: &String) -> String {
    // Open the actual file described in the dir
    let mut actual_file = match File::open(&file_dir) {
        Ok(f) => f,
        Err(e) => panic!("Error occurred opening file: {} - Err: {}", &file_dir, e)
    };

    // Retrieve the contents of the file
    let mut file_content = String::new();
    match actual_file.read_to_string(&mut file_content) {
        Ok(s) => s,
        Err(e) => panic!("Error Reading file: {}", e)
    };

    file_content
}

pub fn should_rsa_encrypt_data(data: Vec<u8>) -> Vec<u8> {
    let public_key_pem = load_service_config().public_key_location;

    if public_key_pem.is_empty() {
        // return data
        return data
    } else {
        // Encrypt with public key
        let rsa = Rsa::public_key_from_pem(read_file_content_from_dir(&public_key_pem).as_bytes()).unwrap();
        let mut buf: Vec<u8> = vec![0; rsa.size() as usize];
        let _ = rsa.public_encrypt(&data, &mut buf, Padding::PKCS1).unwrap();
        println!("Encrypted: {:?}", &buf);
        return buf
    }
}
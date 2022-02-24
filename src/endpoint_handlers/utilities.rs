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
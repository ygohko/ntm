mod error;
mod file_path_producer;
mod object_store;

use std::fs;
use hex_string::HexString;
use sha2::Digest;
use sha2::Sha256;

use crate::file_path_producer::FilePathProducer;
use crate::object_store::ObjectStore;

fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    // ADHOC
    fs::create_dir_all("NTM/Backups")?;
    fs::create_dir_all("NTM/Objects")?;
    let store = ObjectStore::new(&"NTM/Objects");

    let mut producer = FilePathProducer::new(".".to_string());
    let mut done = false;
    while !done {
        let mut path = "".to_string();
        path = match producer.next() {
            Ok(path) => path,
            Err(error) => {
                if error.code == error::CODE_PRODUCING_FINISHED {
                    done = true;
                }
                else {
                    panic!();
                }
                "".to_string()
            },
        };

        if !done {
            let bytes = match fs::read(path) {
                Ok(bytes) => bytes,
                Err(_) => panic!(),
            };
            
            let mut id_bytes = b"b,".to_vec();
            id_bytes = [id_bytes, bytes.clone()].concat();
            println!("id_bytes.len(): {}", id_bytes.len());

            let id = object_id(&id_bytes);
            store.add(&id, &bytes);
        }
    }
    
    /*
    let path = "src/main.rs";
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => panic!(),
    };
    */

    /*
    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(0x00);
    bytes.push(0x01);
    bytes.push(0x02);
    bytes.push(0x03);
    bytes.push(0x04);
    bytes.push(0x05);
    bytes.push(0x06);
    bytes.push(0x07);
     */
    /*
    let mut id_bytes = b"b,".to_vec();
    id_bytes = [id_bytes, bytes.clone()].concat();
    println!("id_bytes.len(): {}", id_bytes.len());

    let id = object_id(&id_bytes);
    store.add(&id, &bytes);
    */

    Ok(())
}

fn object_id(bytes: &Vec<u8>) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(bytes.clone());
    let hash = sha256.finalize();
    let hash_values = hash.to_vec();
    let hex = HexString::from_bytes(&hash_values);
    let result = hex.as_string();

    result
}

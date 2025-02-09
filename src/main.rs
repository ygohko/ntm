mod error;
mod file_path_producer;
mod object_store;

use std::fs;
use std::path;
use std::path::PathBuf;
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
    let date_time = "YYYYMMDDhhmm".to_string();

    let mut producer = FilePathProducer::new(".".to_string());
    let mut done = false;
    while !done {
        let path = match producer.next() {
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
            println!("path: {}", path);

            let bytes = match fs::read(path.clone()) {
                Ok(bytes) => bytes,
                Err(_) => panic!(),
            };
            

            let mut id_bytes = b"b,".to_vec();
            id_bytes = [id_bytes, bytes.clone()].concat();
            println!("id_bytes.len(): {}", id_bytes.len());

            let id = object_id(&id_bytes);
            store.add(&id, &bytes);

            
            
            // TODO: Write reference files.
            let mut reference_path = PathBuf::new();
            reference_path.push("NTM/Backups");
            reference_path.push(date_time.clone());
            reference_path.push(reference_directories(&path));
            match fs::create_dir_all(reference_path.clone()) {
                Ok(_) => (),
                Err(_) => panic!(),
            };
            reference_path.push(reference_file(&path));

            println!("reference_path: {}", reference_path.display());

            match fs::write(reference_path, id) {
                Ok(_) => (),
                Err(_) => panic!(),
            };
            

        }
    }

    Ok(())
}

fn object_id(bytes: &Vec<u8>) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(bytes.clone());
    let hash = sha256.finalize();
    let hash_values = hash.to_vec();
    let hex = HexString::from_bytes(&hash_values);

    hex.as_string()
}

fn reference_directories(path: &str) -> String {
    let mut split: Vec<_> = path.split(path::MAIN_SEPARATOR_STR).collect();
    if split.len() < 1 {
        return "".to_string();
    }
    split.pop();

    split.join(path::MAIN_SEPARATOR_STR)
}

fn reference_file(path: &str) -> String {
    let mut split: Vec<_> = path.split(path::MAIN_SEPARATOR_STR).collect();
    if split.len() < 1 {
        return "".to_string();
    }

    split.pop().unwrap().to_string()
}

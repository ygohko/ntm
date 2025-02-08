mod object_store;

use std::fs;

use crate::object_store::ObjectStore;

fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    // ADHOC
    fs::create_dir_all("NTM/Backups")?;
    fs::create_dir_all("NTM/Objects")?;
    let store = ObjectStore::new(&"NTM/Objects");
    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(0x00);
    bytes.push(0x01);
    bytes.push(0x02);
    bytes.push(0x03);
    bytes.push(0x04);
    bytes.push(0x05);
    bytes.push(0x06);
    bytes.push(0x07);
    store.add("01234567", &bytes);

    Ok(())
}

mod object_store;

use std::fs;

use crate::object_store::ObjectStore;

fn main() -> std::io::Result<()>{
    println!("Hello, world!");

    // ADHOC
    fs::create_dir_all("NTM/Backups")?;
    fs::create_dir_all("NTM/Objects" )?;
    let store = ObjectStore::new(&"NTM/Objects");

    Ok(())
}

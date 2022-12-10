use std::{collections::VecDeque, io::Read};
use sha2::Digest;

fn main() -> () {
    let cwd = std::fs::read_dir(".").unwrap();
    let mut remaining : VecDeque<String> = VecDeque::new();
    
    for elt in cwd {
        let path = elt.unwrap().path();
        let p = path.to_str().unwrap();
        remaining.push_back(String::from(p));
    }

    let mut hashes = std::collections::HashMap::<String, String>::new();

    loop {
        if remaining.is_empty() {
            break;
        }

        let elt = remaining.pop_front().unwrap();

        let path = std::path::Path::new(&elt);

        if path.is_dir() {
            let dirs = std::fs::read_dir(elt.clone()).unwrap();
            for subdir in dirs {
                let s = subdir.unwrap().path();
                let p = s.to_str().unwrap();
                let r = String::from(p);
                remaining.push_back(r);
            }
        } else if path.is_file() {
            let mut hasher = sha2::Sha256::new();
            let mut reader = std::fs::File::open(path).unwrap();
            let mut buf = [0u8; 4096];
            loop {
                let sz = match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(sz) => sz,
                    Err(err) => { println!("Error reading: {}", err); break; }
                };
                let slice = &buf[0..sz];
                hasher.update(slice);
            }
            let hash = hex::encode(hasher.finalize());
            if hashes.contains_key(&hash) {
                let hashpath = hashes.get(&hash).unwrap();
                let stat1 = std::path::Path::new(hashpath);
                let mtime1 = stat1.metadata().unwrap().modified().unwrap();
                let mtime2 = path.metadata().unwrap().modified().unwrap();
                if mtime2 < mtime1 {
                    hashes.insert(hash, elt);
                } else {
                    eprintln!("Ignoring duplicate\n{}\nof older file\n{}", elt, hashpath);
                }
            } else {
                hashes.insert(hash, elt);
            }
        }
    }

    for (_, path) in hashes {
        println!("{}", path);
    }
}
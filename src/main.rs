use std::{collections::VecDeque, io::Read};
use sha2::Digest;

struct FileInfo {
    size : u64,
    path : String
}
fn main() -> () {
    let cwd = std::fs::read_dir(".").unwrap();
    let mut remaining : VecDeque<String> = VecDeque::new();
    
    for elt in cwd {
        let path = elt.unwrap().path();
        let p = path.to_str().unwrap();
        remaining.push_back(String::from(p));
    }

    let mut hashes = std::collections::HashMap::<String, FileInfo>::new();
    let mut file_count = 0;
    loop {
        if remaining.is_empty() {
            break;
        }
        file_count = file_count + 1;
        if file_count % 1000 == 0 {
            eprint!(".");
        }

        let elt = remaining.pop_front().unwrap();

        let path = std::path::Path::new(&elt);

        if path.is_dir() && !path.is_symlink() {
            let dirs = std::fs::read_dir(elt.clone()).unwrap();
            for subdir in dirs {
                let s = subdir.unwrap().path();
                let p = s.to_str().unwrap();
                let r = String::from(p);
                remaining.push_back(r);
            }
        } else if path.is_file() && !path.is_symlink() {
            let mut hasher = sha2::Sha256::new();
            let mut reader = std::fs::File::open(path).unwrap();
            let mut buf = [0u8; 8192];

            let sz = match reader.read(&mut buf) {
                Ok(0) => continue,
                Ok(sz) => sz,
                Err(err) => { eprintln!("Error reading: {}", err); continue; }
            };
            let slice = &buf[0..sz];
            hasher.update(slice);
            let hash = hex::encode(hasher.finalize());
            let next_path_metadata = path.metadata().unwrap();
            let next_path_len = next_path_metadata.len();

            if hashes.contains_key(&hash) {
                let existing_file_info = hashes.get(&hash).unwrap();
                let existing_file_path = std::path::Path::new(&existing_file_info.path);
                let existing_file_mtime = existing_file_path.metadata().unwrap().modified().unwrap();
                let next_file_mtime = next_path_metadata.modified().unwrap();

                // Record the oldest file.
                if next_file_mtime < existing_file_mtime && existing_file_info.size == next_path_len {
                    hashes.insert(hash, FileInfo{path: elt, size: next_path_len});
                } else {
                    eprintln!("Ignoring duplicate {} of older file {}", elt, existing_file_info.path);
                }
            } else {
                hashes.insert(hash, FileInfo{path: elt, size: next_path_len});
            }
        }
    }

    for (_, path) in hashes {
        println!("{}", path.path);
    }
}

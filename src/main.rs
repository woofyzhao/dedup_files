mod thread_pool;

use walkdir::WalkDir;
use std::{env, fs, io};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};
use base64ct::{Base64, Encoding};
use crate::thread_pool::ThreadPool;

#[derive(Debug, Clone)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    digest: String,
}

fn digest(mut file: fs::File) -> (u64, String) {
    let mut hasher = Sha256::new();
    let n = io::copy(&mut file, &mut hasher).unwrap();
    println!("{} bytes copied", n);
    let hash = hasher.finalize();
    (n, Base64::encode_string(&hash))
}

fn walk_and_digest(dir: &str, threads: u8) -> Vec<FileInfo> {
    let pool = ThreadPool::new(threads as usize);
    let result = Arc::new(Mutex::new(vec![]));

    for entry in WalkDir::new(dir) {
        let file = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error get dir entry fail, {}", e);
                continue
            },
        };

        println!("Found {:?}", file.path());
        if !file.file_type().is_file() {
            continue;
        }

        let res = Arc::clone(&result);
        pool.execute(move || {
            let handle = match fs::File::open(file.path()) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Error: open {:?} fail, {}", file.path(), e);
                    return
                }
            };

            let (size, digest) = digest(handle);

            res.lock().unwrap().push(FileInfo {
                path: file.path().to_path_buf(),
                size,
                digest,
            });
        });
    }
    drop(pool);
    let v = result.lock().unwrap();
    v.to_vec()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: {} <dir> <threads>", args[0]);
        exit(-1)
    }

    let dir= &args[1];
    let threads = args[2].parse::<u8>().unwrap();

    let info_list = walk_and_digest(dir, threads);
    println!("{:?}", info_list);

    let mut map = HashMap::new();
    for item in info_list {
        let v = map.entry(item.digest.clone()).or_insert(Vec::new());
        v.push(item);
    }

    let mut saved = 0u64;
    for mut v in map.into_values() {
        if v.len() < 2 {
            continue
        }
        saved += v[0].size * (v.len() as u64 - 1);
        v.sort_by(|a, b| a.path.cmp(&b.path));
        println!("{}:{}", v.len(), v.iter().map(|x| x.path.to_str().unwrap().to_string())
            .collect::<Vec<String>>().join(","));

        for item in &v[1..] {
            println!("deleting {:?}", item.path);
            fs::remove_file(item.path.as_path()).unwrap();
        }
    }

    println!("total saved = {}", saved);
}

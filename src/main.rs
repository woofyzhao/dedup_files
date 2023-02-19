use walkdir::WalkDir;
use std::{fs, io};
use std::collections::HashMap;
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use base64ct::{Base64, Encoding};

#[derive(Debug)]
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

fn walk(dir: &str) -> Vec<FileInfo> {
    let mut result = vec![];
    for entry in WalkDir::new(dir) {
        let file = entry.expect("traverse dir fail");
        println!("found {:?}", file.file_name());
        if !file.file_type().is_file() {
            continue;
        }
        let handle = fs::File::open(file.path()).expect("cannot open file");
        let (size, digest) = digest(handle);
        result.push(FileInfo {
            path: file.path().to_path_buf(),
            size,
            digest,
        });
    }
    result
}

fn main() {
    let dir = "/Volumes/My Passport/腾讯电影协会/电子书/";
    let info_list = walk(dir);
    //println!("{:?}", info_list);

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
            //fs::remove_file(item.path.as_path()).unwrap();
        }
    }

    println!("total saved = {}", saved);
}

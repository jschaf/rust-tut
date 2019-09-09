#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::{error, fmt, fs, io};

use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug)]
pub struct KvStore {
    //    reader: File,
    writer: File,
    map: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Set(String, String),
    Remove(String),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Command::Set(key, value) => write!(f, "Set({}, {})", key, value),
            Command::Remove(key) => write!(f, "Remove({})", key),
        }
    }
}

#[derive(Debug)]
pub enum KvsError {
    Encoding(serde_json::error::Error),
    Io(io::Error),
    KeyNotFound,
}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            KvsError::Encoding(err) => write!(f, "Encoding error: {}", err),
            KvsError::Io(err) => write!(f, "IO error: {}", err),
            KvsError::KeyNotFound => write!(f, "Key not found"),
        }
    }
}

impl error::Error for KvsError {
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl From<serde_json::error::Error> for KvsError {
    fn from(err: serde_json::error::Error) -> Self {
        KvsError::Encoding(err)
    }
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> Self {
        KvsError::Io(err)
    }
}

/// An in-memory key-value (KV) store.
///
/// # Examples
///
/// ```
/// use kvs::KvStore;
///
/// let mut store = KvStore::new();
/// store.set(String::from("a"), String::from("alpha"));
/// assert_eq!(store.get(String::from("a")), Some("alpha".to_string()));
///
/// store.remove(String::from("a"));
/// assert_eq!(store.get(String::from("a")), None);
/// ```
impl KvStore {
    pub fn open(dir: impl Into<PathBuf>) -> Result<KvStore, KvsError> {
        let dir = dir.into();
        if !dir.is_dir() {
            panic!(
                "KvStore.open() must receive a directory but had '{:?}'.",
                dir
            );
        }
        eprintln!("\n# Opening kvstore in dir {:?}", dir);
        let map = KvStore::replay_logs_in_dir(dir.as_path())?;
        eprintln!("  Map after replay: {:?}", map);

        let log_file = KvStore::next_log_file_in_dir(dir.as_path())?;
        eprintln!("  Next log file: {:?}", log_file);
        let writer = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(log_file)?;

        Ok(KvStore { writer, map })
    }

    fn replay_logs_in_dir(path: &Path) -> Result<HashMap<String, String>, KvsError> {
        eprintln!("Replaying logs in {:?}", path);
        let mut map = HashMap::new();
        let mut logs = KvStore::collect_log_files(path)?;
        logs.sort_by_key(|p| KvStore::extract_log_number(p).expect("Kvslog must have u32"));
        eprintln!("  Sorted logs: {:?}", logs);
        for log in logs {
            KvStore::replay_log(log.as_path(), &mut map);
        }
        Ok(map)
    }

    fn collect_log_files(dir: &Path) -> Result<Vec<PathBuf>, KvsError> {
        eprintln!("  Collecting logs");
        let mut logs = vec![];
        for entry in fs::read_dir(dir).expect("dir should exist") {
            let path = entry?.path();
            eprintln!("    Found path {:?}", path);
            if path
                .extension()
                .and_then(|p| p.to_str())
                .map_or(false, |p| p == "kvslog")
            {
                eprintln!("      Pushing path {:?}", path);
                logs.push(path);
            }
        }
        Ok(logs)
    }

    fn next_log_file_in_dir(path: &Path) -> Result<PathBuf, KvsError> {
        eprintln!("  Next log in dir {:?}", path);
        let mut log_nums = vec![];
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            eprintln!("  Found path {:?}", path);
            if path.ends_with(".log") {
                log_nums.push(KvStore::extract_log_number(path.as_path()).unwrap());
            }
        }
        let n = log_nums.iter().max().unwrap_or(&0u32);
        Ok(PathBuf::from(path).join(format!("log-{}.kvslog", n)))
    }

    fn extract_log_number(path: &Path) -> Option<u32> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^.*-(\d+)\.kvlog$").unwrap();
        }
        let p = path.to_str().expect("Move Path to str");
        RE.captures(p)
            .and_then(|captures| captures.get(0))
            .map(|m| {
                m.as_str()
                    .parse::<u32>()
                    .expect("Kvslog path must parse as u32")
            })
    }

    fn replay_log(path: &Path, map: &mut HashMap<String, String>) -> Result<(), KvsError> {
        eprintln!("  Replaying single log: {:?}", path);
        let file = fs::OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.expect("Expected valid line");
            let cmd: Command = serde_json::from_slice(line.as_bytes())?;
            match cmd {
                Command::Set(key, value) => map.insert(key, value),
                Command::Remove(key) => map.remove(key.as_str()),
            };
        }
        Ok(())
    }

    /// Returns a clone of the value corresponding to the key.
    pub fn get(&self, key: String) -> Result<Option<String>, KvsError> {
        Ok(self.map.get(key.as_str()).cloned())
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did have this key present, the value is updated.
    pub fn set(&mut self, key: String, value: String) -> Result<(), KvsError> {
        eprintln!("# Setting key: {} to value: {}", key, value);
        // TODO: buffer writes.
        let bytes = serde_json::to_vec(&Command::Set(key.clone(), value.clone()))?;
        //        write!(&self.writer, "{}", bytes.len() as u32);
        self.writer.write(bytes.as_slice())?;
        self.writer.write("\n".as_bytes());
        self.writer.flush();
        self.map.insert(key, value);
        Ok(())
    }

    /// Removes a key from the map.
    pub fn remove(&mut self, key: String) -> Result<(), KvsError> {
        eprintln!("Removing key: {}", key);
        let bytes = serde_json::to_vec(&Command::Remove(key.clone()))?;
        //        write!(&self.writer, "{}", bytes.len() as u32);
        self.writer.write(bytes.as_slice())?;
        self.writer.write("\n".as_bytes());
        self.writer.flush();
        eprintln!("rm map {:?}", self.map);
        if self.map.remove(key.as_str()).is_none() {
            return Err(KvsError::KeyNotFound);
        }
        return Ok(());
    }
}

pub fn list_dir(p: &Path) {
    println!("list_dir: Current dir: {:?}", p);
    let paths = fs::read_dir(p).unwrap();
    for path in paths {
        println!("  Name: {}", path.unwrap().path().display())
    }
}

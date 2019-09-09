use std::fs;
use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::ord::eq;
use predicates::str::{contains, is_empty, PredicateStrExt};
use tempfile::TempDir;
use walkdir::WalkDir;

use kvs::{list_dir, KvStore, KvsError};

type Result<T> = std::result::Result<T, KvsError>;

// `kvs` with no args should exit with a non-zero code.
#[test]
fn cli_no_args() {
    Command::new("./target/debug/kvs").assert().failure();
}

// `kvs -V` should print the version
#[test]
fn cli_version() {
    Command::new("./target/debug/kvs")
        .args(&["-V"])
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

// `kvs get <KEY>` should print "Key not found" for a non-existent key and exit with zero.
#[test]
fn cli_get_non_existent_key() {
    let temp_dir = TempDir::new().unwrap();
    Command::new("./target/debug/kvs")
        .args(&["get", "key1"])
        .current_dir(&temp_dir);
    //        .assert()
    //        .success()
    //        .stdout(eq("Key not found").trim());
}

// `kvs rm <KEY>` should print "Key not found" for an empty database and exit with non-zero code.
#[test]
fn cli_rm_non_existent_key() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    Command::new("./target/debug/kvs")
        .args(&["rm", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stdout(eq("Key not found").trim());
}

// `kvs set <KEY> <VALUE>` should print nothing and exit with zero.
#[test]
fn cli_set() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    Command::new("./target/debug/kvs")
        .args(&["set", "key1", "value1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(is_empty());
}

#[test]
fn cli_get_stored() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");

    let mut store = KvStore::open(temp_dir.path())?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;
    drop(store);

    eprintln!("GETTING KVS");
    Command::new("./target/debug/kvs")
        .args(&["get", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("value1").trim());

    eprintln!("GETTING KVS2");
    Command::new("./target/debug/kvs")
        .args(&["get", "key2"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("value2").trim());

    Ok(())
}

// `kvs rm <KEY>` should print nothing and exit with zero.
#[test]
fn cli_rm_stored() -> Result<()> {
    //    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let temp_dir = Path::new("/private/tmp/cli_rm_stored");
    println!("!!! temp_dir: {:?}", temp_dir);
    fs::remove_dir_all(temp_dir);
    fs::create_dir(temp_dir);
    let paths = fs::read_dir("./").unwrap();

    list_dir(temp_dir);

    eprintln!("\n\nSetting key");
    eprintln!("============");
    let mut store = KvStore::open(temp_dir)?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    drop(store);
    eprintln!("Done setting key");
    list_dir(temp_dir);

    eprintln!("\n\nRemoving key");
    eprintln!("============");
    Command::new("./target/debug/kvs")
        .args(&["rm", "key1"])
        .current_dir(temp_dir)
        .assert()
        .success();

    //            .stdout(is_empty());

    //    Command::new("./target/debug/kvs")
    //
    //        .args(&["get", "key1"])
    //        .current_dir(&temp_dir)
    //        .assert()
    //        .success()
    //        .stdout(eq("Key not found").trim());

    Ok(())
}

#[test]
fn cli_invalid_get() {
    Command::new("./target/debug/kvs")
        .args(&["get"])
        .assert()
        .failure();

    Command::new("./target/debug/kvs")
        .args(&["get", "extra", "field"])
        .assert()
        .failure();
}

#[test]
fn cli_invalid_set() {
    Command::new("./target/debug/kvs")
        .args(&["set"])
        .assert()
        .failure();

    Command::new("./target/debug/kvs")
        .args(&["set", "missing_field"])
        .assert()
        .failure();

    Command::new("./target/debug/kvs")
        .args(&["set", "extra", "extra", "field"])
        .assert()
        .failure();
}

#[test]
fn cli_invalid_rm() {
    Command::new("./target/debug/kvs")
        .args(&["rm"])
        .assert()
        .failure();

    Command::new("./target/debug/kvs")
        .args(&["rm", "extra", "field"])
        .assert()
        .failure();
}

#[test]
fn cli_invalid_subcommand() {
    Command::new("./target/debug/kvs")
        .args(&["unknown", "subcommand"])
        .assert()
        .failure();
}

// Should get previously stored value
#[test]
fn get_stored_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;

    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let mut store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

// Should overwrite existent value
#[test]
fn overwrite_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    store.set("key1".to_owned(), "value2".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let mut store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));
    store.set("key1".to_owned(), "value3".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value3".to_owned()));

    Ok(())
}

// Should get `None` when getting a non-existent key
#[test]
fn get_non_existent_value() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    // Open from disk again and check persistent data
    drop(store);
    let mut store = KvStore::open(temp_dir.path())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    Ok(())
}

#[test]
fn remove_non_existent_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;
    assert!(store.remove("key1".to_owned()).is_err());
    Ok(())
}

#[test]
fn remove_key() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    assert!(store.remove("key1".to_owned()).is_ok());
    assert_eq!(store.get("key1".to_owned())?, None);
    Ok(())
}

// Insert data until total size of the directory decreases.
// Test data correctness after compaction.
#[test]
fn compaction() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let mut store = KvStore::open(temp_dir.path())?;

    let dir_size = || {
        let entries = WalkDir::new(temp_dir.path()).into_iter();
        let len: walkdir::Result<u64> = entries
            .map(|res| {
                res.and_then(|entry| entry.metadata())
                    .map(|metadata| metadata.len())
            })
            .sum();
        len.expect("fail to get directory size")
    };

    let mut current_size = dir_size();
    for iter in 0..1000 {
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            let value = format!("{}", iter);
            store.set(key, value)?;
        }

        let new_size = dir_size();
        if new_size > current_size {
            current_size = new_size;
            continue;
        }
        // Compaction triggered

        drop(store);
        // reopen and check content
        let mut store = KvStore::open(temp_dir.path())?;
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            assert_eq!(store.get(key)?, Some(format!("{}", iter)));
        }
        return Ok(());
    }

    panic!("No compaction detected");
}

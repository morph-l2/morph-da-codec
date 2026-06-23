use std::{fs, path::Path};

use morph_da_decoder_core::decompress_morph_da_zstd;

const BLOB_WIDTH: usize = 4096;

#[test]
fn decompresses_batch() {
    let batches_dir = Path::new("../../../testdata/batches");
    let batch_dirs = dir_entries_sorted(batches_dir);
    assert!(
        !batch_dirs.is_empty(),
        "no batch directories found in {batches_dir:?}"
    );

    for batch_dir in &batch_dirs {
        let blob_paths = blob_paths_sorted(batch_dir);
        assert!(
            !blob_paths.is_empty(),
            "no blob hex files found in {batch_dir:?}"
        );

        let batch_data: Vec<u8> = blob_paths
            .iter()
            .flat_map(|p| decode_blob(&read_hex(p)))
            .collect();

        let decoded = decompress_morph_da_zstd(&batch_data).unwrap();
        println!(
            "batch {:?} decoded len {:?}",
            batch_dir.file_name().unwrap_or_default(),
            decoded.len()
        );
    }
}

fn dir_entries_sorted(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    entries.sort();
    entries
}

fn blob_paths_sorted(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file() && blob_index(p).is_some())
        .collect();
    entries.sort_by_key(|p| blob_index(p));
    entries
}

fn blob_index(path: &Path) -> Option<usize> {
    path.file_name()
        .and_then(|name| name.to_str())?
        .strip_prefix("blob-")?
        .strip_suffix(".hex")?
        .parse()
        .ok()
}

fn read_hex(path: &Path) -> Vec<u8> {
    let raw = fs::read_to_string(path).unwrap();
    let hex = raw.trim();
    hex::decode(hex.strip_prefix("0x").unwrap_or(hex)).unwrap()
}

fn decode_blob(blob: &[u8]) -> Vec<u8> {
    assert_eq!(blob.len(), BLOB_WIDTH * 32, "invalid blob size");

    let mut out = vec![0u8; BLOB_WIDTH * 31];
    for (i, chunk) in blob.chunks_exact(32).enumerate() {
        assert_eq!(chunk[0], 0, "non-zero high byte in field element {i}");
        out[i * 31..i * 31 + 31].copy_from_slice(&chunk[1..]);
    }
    out
}

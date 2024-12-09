use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

#[test]
fn compare_diff_files() {
    let metadata1 = fs::symlink_metadata(Path::new("/home/coder/.npmrc_bak")).unwrap();
    let metadata2 =
        fs::symlink_metadata(Path::new("/home/coder/dup-file-remover/README.md")).unwrap();
    let metadata3 = fs::symlink_metadata(Path::new("/etc/hosts")).unwrap();
    let metadata4 = fs::symlink_metadata(Path::new("/.dockerenv")).unwrap();
    println!("metadata1: {:?}, dev={:?}", metadata1, metadata1.dev());
    println!("metadata2: {:?}, dev={:?}", metadata2, metadata2.dev());
    println!("metadata3: {:?}, dev={:?}", metadata3, metadata3.dev());
    println!("metadata4: {:?}, dev={:?}", metadata4, metadata4.dev());
}

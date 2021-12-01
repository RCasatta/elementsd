use bitcoin_hashes::{sha256, Hash};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::str::FromStr;
use tar::Archive;

include!("src/versions.rs");

//  https://github.com/ElementsProject/elements/releases/download/elements-0.21.0/elements-elements-0.21.0-x86_64-linux-gnu.tar.gz
//  https://github.com/ElementsProject/elements/releases/download/elements-0.18.1.12/elements-0.18.1.12-x86_64-linux-gnu.tar.gz

// https://github.com/ElementsProject/elements/releases/download/elements-0.18.1.12/elements-0.18.1.12-osx64.tar.gz

#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", target_arch = "aarch64"),
))]
fn download_filename() -> String {
    format!("elements-{}-osx64.tar.gz", &VERSION)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn download_filename() -> String {
    format!("elements-{}-x86_64-linux-gnu.tar.gz", &VERSION)
}

fn get_expected_sha256(filename: &str) -> Result<sha256::Hash, ()> {
    let sha256sums_filename = format!("sha256/SHA256SUMS_{}.asc", &VERSION);
    let file = File::open(sha256sums_filename).map_err(|_| ())?;
    for line in BufReader::new(file).lines().flatten() {
        let tokens: Vec<_> = line.split("  ").collect();
        if tokens.len() == 2 && filename == tokens[1] {
            return sha256::Hash::from_str(tokens[0]).map_err(|_| ());
        }
    }
    Err(())
}

fn main() {
    if !HAS_FEATURE {
        return;
    }
    let download_filename = download_filename();
    println!("{}", download_filename);
    let expected_hash = get_expected_sha256(&download_filename).unwrap();
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let elements_exe_home = Path::new(&out_dir).join("elements");
    if !elements_exe_home.exists() {
        std::fs::create_dir(&elements_exe_home).unwrap();
    }
    let existing_filename = elements_exe_home
        .join(format!("elements-{}", VERSION))
        .join("bin")
        .join("elementsd");

    if !existing_filename.exists() {
        println!(
            "filename:{} version:{} hash:{}",
            download_filename, VERSION, expected_hash
        );

        let version = if cfg!(feature = "0_21_0") {
            "0.21.0"
        } else {
            VERSION
        };

        let url = format!(
            "https://github.com/ElementsProject/elements/releases/download/elements-{}/{}",
            version, download_filename
        );
        println!("url:{}", url);
        let mut downloaded_bytes = Vec::new();

        let _size = ureq::get(&url)
            .call()
            .into_reader()
            .read_to_end(&mut downloaded_bytes)
            .unwrap();

        let downloaded_hash = sha256::Hash::hash(&downloaded_bytes);
        assert_eq!(expected_hash, downloaded_hash);
        let d = GzDecoder::new(&downloaded_bytes[..]);

        let mut archive = Archive::new(d);
        for mut entry in archive.entries().unwrap().flatten() {
            if let Ok(file) = entry.path() {
                if file.ends_with("elementsd") {
                    entry.unpack_in(&elements_exe_home).unwrap();
                }
            }
        }
    }
}

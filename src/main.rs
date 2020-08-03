
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Write, BufRead, BufReader, BufWriter};
use std::path::Path;

use anyhow::Result;
use clap::{App, Arg, SubCommand};
use git2::{Oid, Repository};
use walkdir::WalkDir;

const PT_HASH_FILE: &'static str = "pt_commit_hash.txt";

fn main() -> Result<()> {
    let matches = App::new("Pt Commit Tool")
        .version("0.1")
        .subcommand(
            SubCommand::with_name("record")
                .about("record pt repo hash")
                .arg(
                    Arg::with_name("pt-root-path")
                        .short("s")
                        .long("pt-root-path")
                        .required(true)
                        .help("Specify pt root path")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("pi-pt-root-path")
                        .short("t")
                        .long("pi-pt-root-path")
                        .help("Specify pi pt root path")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("restore")
                .about("restore pt repo hash")
                .arg(
                    Arg::with_name("pi-pt-root-path")
                        .short("s")
                        .long("pi-pt-root-path")
                        .required(true)
                        .help("Specify pt root path")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("pt-root-path")
                        .short("t")
                        .long("pt-root-path")
                        .required(true)
                        .help("Specify pi pt root path")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(subcommand_matches) = matches.subcommand_matches("record") {
        let pt_root = subcommand_matches
            .value_of("pt-root-path")
            .expect("You must specify pt root path");

        let pi_pt_root = subcommand_matches
            .value_of("pi-pt-root-path")
            .expect("You must specify pi pt root path");

        record_pt_repo_hash(pt_root, pi_pt_root)?
    }

    if let Some(subcommand_matches) = matches.subcommand_matches("restore") {
        let pt_root = subcommand_matches
            .value_of("pt-root-path")
            .expect("You must specify pt root path");

        let pi_pt_root = subcommand_matches
            .value_of("pi-pt-root-path")
            .expect("You must specify pi pt root path");

        restore_pt_by_hash(pt_root, pi_pt_root)?
    }

    Ok(())
}

// 通过pi_pt根目录的pt_commit_hash.txt文件把pt中的rust库恢复到指定的commit hash
fn restore_pt_by_hash<P: AsRef<Path>>(pt_path: P, pi_pt_path: P) -> Result<()> {
    let path = Path::new(pi_pt_path.as_ref()).join(PT_HASH_FILE);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let mut info = line.split(":");
        let (repo_name, hash) = (info.next().unwrap(), info.next().unwrap());
        let path = Path::new(pt_path.as_ref()).join(repo_name);
        let repo = Repository::open(path)?;
        let oid = Oid::from_str(hash)?;
        repo.set_head_detached(oid)?;
    }
    Ok(())
}

// 把pt中rust库的当前的commit hash 写入到pi_pt根目录中的 pt_commit_hash.txt 文件中
fn record_pt_repo_hash<P: AsRef<Path>>(pt_path: P, pi_pt_path: P) -> Result<()> {
    let path = Path::new(pi_pt_path.as_ref()).join(PT_HASH_FILE);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    let mut writer = BufWriter::new(file);

    for entry in WalkDir::new(pt_path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let repo_name = entry.path().components().last().unwrap().as_os_str();
        let repo = Repository::open(entry.path())?;

        let head = repo.head()?;
        let head_ref = head.name().unwrap();

        let mut line = String::new();
        line += repo_name.to_str().unwrap();
        line += ":";
        line += repo
            .revparse_single(head_ref)?
            .id()
            .to_string()
            .as_str();
        line += "\n";

        writer.write(line.as_bytes())?;
    }

    writer.flush()?;

    Ok(())
}

use clap::Parser;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use lopdf::dictionary;
use lopdf::Dictionary;
use lopdf::Document;
use lopdf::Object;
use lopdf::Stream;
use tar::Builder;
use walkdir::WalkDir;
use zstd::Encoder;

fn add_metadata_to_pdf(input_path: &str, output_path: &Path, project_folder: &Path) {
    let mut doc = Document::load(input_path).unwrap();

    // Create a new dictionary for the metadata
    let mut info = dictionary! {};

    info.set(
        "PixiToml",
        Object::string_literal(std::fs::read_to_string(project_folder.join("pixi.toml")).unwrap()),
    );
    info.set(
        "PixiLock",
        Object::string_literal(std::fs::read_to_string(project_folder.join("pixi.lock")).unwrap()),
    );

    // create zst archive from project folder

    let tempdir = tempfile::tempdir().unwrap();
    let data_path = tempdir.path().join("project.zst");
    compress_folder_to_zstd_archive(project_folder, &data_path).unwrap();
    let data = fs::read(&data_path).unwrap();
    let stream = Stream::new(Dictionary::new(), data).with_compression(false);
    // info.set("Data", Object::Stream(stream));

    let object_id = doc.add_object(Object::Stream(stream));
    info.set("DataFolder", object_id);

    doc.trailer.set("Pixi", Object::Dictionary(info));

    // Save the modified PDF
    doc.save(output_path).unwrap();
}

fn read_metadata_from_pdf(input_path: &Path, output_folder: &Path) -> Vec<(String, String)> {
    let doc = Document::load(input_path).unwrap();
    let metadata = Vec::new();

    let trailer = doc.trailer.as_hashmap();

    let pixi_info = trailer
        .get("Pixi".as_bytes())
        .and_then(|obj| obj.as_dict().ok());
    // println!("trailer: {:?}", trailer);
    for (_key, value) in pixi_info.unwrap() {
        if let Ok(_value) = value.as_str() {
            // println!(
            //     "{} -> {}",
            //     String::from_utf8_lossy(key),
            //     String::from_utf8_lossy(value)
            // );
        } else {
            // println!("{} -> could not stringify", String::from_utf8_lossy(key));
            let id = value.as_reference().unwrap();
            let stream = doc.get_object(id).unwrap().as_stream().unwrap();
            let data = stream.content.clone();

            let tempdir = tempfile::tempdir().unwrap();
            let data_path = tempdir.path().join("project.zst");

            let mut file = File::create(&data_path).unwrap();
            file.write_all(&data).unwrap();
            file.flush().unwrap();
            extract_to_folder(&data_path, output_folder).unwrap();
        }
    }

    // if let Some(info) = trailer.get("Info").and_then(|obj| obj.as_dict().ok()) {
    //     for (key, value) in info {
    //         if let Some(value) = value.as_str().ok() {
    //             metadata.push((key.to_string(), value.to_string()));
    //         }
    //     }
    // }

    metadata
}

fn extract_to_folder(data_path: &Path, output_folder: &Path) -> io::Result<()> {
    println!("extracting {:?} to {:?}", data_path, output_folder);
    let data_file = File::open(data_path)?;
    let decoder = zstd::Decoder::new(data_file)?;
    let mut tar = tar::Archive::new(decoder);

    tar.unpack(output_folder)?;

    Ok(())
}

fn run(data_path: &Path, args: Vec<String>) -> io::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let output_folder = temp_dir.path();

    read_metadata_from_pdf(data_path, output_folder);

    // run a subprocess `pixi run` with the remaining args
    let status = std::process::Command::new("pixi")
        .arg("run")
        .args(args)
        .current_dir(output_folder)
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "pixi run failed with non-zero exit code",
        ));
    }

    Ok(())
}

fn compress_folder_to_zstd_archive<P: AsRef<Path>>(
    folder_path: P,
    output_path: P,
) -> io::Result<()> {
    let folder_path = folder_path.as_ref();
    let output_file = File::create(output_path)?;
    let encoder = Encoder::new(output_file, 0)?; // 0 is the compression level
    let mut tar_builder = Builder::new(encoder);

    for entry in WalkDir::new(folder_path) {
        let entry = entry?;
        let path = entry.path();

        let rel_path = path.strip_prefix(folder_path).unwrap();
        if rel_path.starts_with(".pixi") {
            continue;
        }

        if path.is_file() {
            tar_builder.append_path_with_name(path, rel_path)?;
        }
    }

    let mut inner = tar_builder.into_inner()?;
    inner.flush()?;
    Ok(())
}

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Your Name")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Embed a pixi project into a pdf
    #[clap(name = "embed")]
    Embed(Embed),

    /// Extract a pixi project from a pdf into a folder
    #[clap(name = "extract")]
    Extract(Extract),

    /// Run a pixi command from a PDF (inside a temporary folder)
    #[clap(name = "run")]
    Run(Run),
}

#[derive(Parser, Debug)]
struct Run {
    #[clap(required = true)]
    file: PathBuf,

    #[clap(last = true)]
    args: Vec<String>,
}

#[derive(Parser, Debug)]
struct Embed {
    #[clap(short, long)]
    file: PathBuf,

    #[clap(short, long)]
    project: PathBuf,

    #[clap(short, long)]
    out_file: PathBuf,
}

#[derive(Parser, Debug)]
struct Extract {
    #[clap(short, long)]
    file: PathBuf,

    #[clap(short, long)]
    out_folder: PathBuf,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Embed(add) => {
            println!(
                "Embedding contents from: {:?} + {:?} to {:?}",
                add.file, add.project, add.out_file
            );
            add_metadata_to_pdf(add.file.to_str().unwrap(), &add.out_file, &add.project);
        }
        SubCommand::Extract(read) => {
            println!(
                "Extracting contents from: {:?} to {:?}",
                read.file, read.out_folder
            );
            let _ = read_metadata_from_pdf(&read.file, &read.out_folder);
        }
        SubCommand::Run(run_opts) => {
            println!(
                "Running pixi project from: {:?} with arguments {:?}",
                run_opts.file, run_opts.args
            );
            run(&run_opts.file, run_opts.args).unwrap();
        }
    }
}

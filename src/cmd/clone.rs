use super::{
    git_protocol::{PackFile, PktLine, PktLines},
    tree::FileTree,
    Error, Result,
};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use std::path::PathBuf;

pub async fn run(url: String, dir: String) -> Result<()> {
    println!("url: {url}, dir: {dir}");
    let root_dir: PathBuf = format!("./{dir}").into();
    if !root_dir.exists() {
        std::fs::create_dir(&root_dir)?;
        super::init::run(&root_dir)?;
    }

    let res = reqwest::get(format!("{url}/info/refs?service=git-upload-pack"))
        .await?
        .bytes()
        .await?;

    let lines = PktLines::from(res);
    let master_ref = get_master_ref(lines).ok_or(Error::from("Cannot find refs/heads/master"))?;

    println!("refs/heads/master: {master_ref}");

    let client = reqwest::Client::new();
    //let mut objects: Vec<GitObject> = vec![];

    let stream = client
        .post(format!("{url}/git-upload-pack"))
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-git-upload-pack-request"),
        )
        .body(format!(
            "{}{}{}{}",
            PktLine::new(
                format!("want {master_ref} multi_ack_detailed side-band-64k\n").into_bytes()
            ),
            PktLine::new(format!("want {master_ref}\n").into_bytes()),
            PktLine::flush(),
            PktLine::new(b"done\n".into()),
        ))
        .send()
        .await?
        .bytes_stream();

    let mut pkt_stream = PktLineStream::new(stream);
    let mut packed_bytes: Vec<u8> = vec![];

    while let Some(line) = pkt_stream.next().await? {
        if line.to_string().as_str() == "0008NAK\n" {
            continue;
        }

        match line.split_first().map(|(first, rest)| (*first, rest)) {
            Some((1, rest)) => {
                packed_bytes.append(&mut rest.to_vec());
            }
            Some((2, rest)) => {
                for progress in format_progress(rest) {
                    println!("{progress}")
                }
            }
            None => {
                println!("[remote] Reached flush line!");
                break;
            }
            _ => {
                return Err(Error::Other(anyhow::anyhow!(
                    "Unexpected pkt line during unpacking pack file. {line}",
                )));
            }
        }
    }

    let objects = PackFile::get_objects(packed_bytes);

    for object in objects.iter() {
        object.write(&root_dir)?;
    }

    let tree = FileTree::new(root_dir, &objects);
    tree.write_all()
}

fn get_master_ref(lines: PktLines) -> Option<String> {
    lines
        .into_iter()
        .map(|line| format!("{line}"))
        .find(|line| line.contains("refs/heads/master"))
        .map(|line| line[4..44].to_string())
}

fn format_progress(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .replace('\r', "\n")
        .trim_end_matches("\n")
        .split("\n")
        .map(|v| v.to_string())
        .collect()
}

#[derive(Debug)]
struct PktLineStream<S>
where
    S: Stream<Item = reqwest::Result<Bytes>> + std::marker::Unpin,
{
    stream: S,
    lines: Option<PktLines>,
}

impl<S> PktLineStream<S>
where
    S: Stream<Item = reqwest::Result<Bytes>> + std::marker::Unpin,
{
    fn new(stream: S) -> Self {
        Self {
            stream,
            lines: Some(PktLines::new(vec![])),
        }
    }

    async fn next(&mut self) -> Result<Option<PktLine>> {
        if let Some(line) = self.lines.as_mut().and_then(PktLines::next) {
            return Ok(Some(line));
        }

        while let Some(result) = self.stream.next().await {
            let bytes = result?;
            let mut lines = self.lines.take().unwrap().append(bytes.to_vec());
            let line = lines.next();
            self.lines = Some(lines);

            if let Some(line) = line {
                return Ok(Some(line));
            }
        }

        Ok(None)
    }
}

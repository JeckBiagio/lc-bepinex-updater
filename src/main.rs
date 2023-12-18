use std::cmp::min;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;
use regex::Regex;

use indicatif::{ ProgressBar, ProgressStyle };
use futures_util::StreamExt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn download_file() -> Result<String> {
    let client = reqwest::Client::new();
    let path = "./BepInEx"; 
    
    let url_pre = "https://docs.google.com/uc?export=download&id=1CJTxpobQuIuTZekptBTjw3jKZsN4UWWv";
    let res_get_id = client
        .get(url_pre)
        .send()
        .await
        .or(Err(format!("Errore durante la prima call")))?;
    
    let id_regex = Regex::new(r".*&amp;confirm=(?<id>\w+)&amp.*").unwrap();
    let text = res_get_id.text().await.expect("Vuoto");

    let Some(caps) = id_regex.captures(&text) else {
        return Ok(format!("Unlucky {}", "aa"));
    };
    println!("Found code: {}", &caps["id"]);
    
    let url = format!("{}{}{}", "https://docs.google.com/uc?export=download&confirm=", &caps["id"], "&id=1CJTxpobQuIuTZekptBTjw3jKZsN4UWWv");
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET")))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length"))?;
    
    println!("Downloading file...");

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
                 .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.yellow/DarkYellow}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                 .progress_chars("▇·"));
    pb.set_message(&format!("Downloading..."));
    
    let mut downloaded: u64 = 0;
    let mut bytes: Vec<u8> = vec![];
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        bytes.append(&mut chunk.to_vec());

        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("Downloaded to {}", path));

    println!("Download finito! Estraggo...");

    let folder_name = PathBuf::from("./BepInEx");
    let content = Cursor::new(bytes); 
    zip_extract::extract(content, &folder_name, true).or(Err(format!("Problemi con il dezip")))?;

    return Ok(format!("Completato con successo!"));
}

#[tokio::main]
async fn main() -> Result<()> {
    let old_dir_path = Path::new("./BepInEx");
    
    if old_dir_path.exists() {
        println!("Vecchia cartella trovata... la rimuovo");
        std::fs::remove_dir_all(old_dir_path).unwrap();
    }
    
    let code = download_file().await;
    match code {
        Ok(s) => println!("Tutto ok: {}", s),
        Err(err) => panic!("Errore: {}", err),
    }

    Ok(())    
}

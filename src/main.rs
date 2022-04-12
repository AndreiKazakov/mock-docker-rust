use std::env;
use std::ffi::CString;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use bytes::Bytes;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, WWW_AUTHENTICATE};
use serde_json::Value;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let image: Vec<&str> = args[2].split(':').collect();
    let command = &args[3];
    let command_args = &args[4..];
    let tmp = env::temp_dir().join("docker-rust-root");
    let cpath = CString::new(tmp.as_os_str().to_str().unwrap()).unwrap();

    fs::create_dir_all(tmp.join("dev")).unwrap();
    fs::File::create(tmp.join("dev").join("null")).unwrap();

    let blobs = get_image_blobs(image[0], image[1])?;

    for bytes in blobs {
        untar(&bytes, tmp.to_str().unwrap());
    }


    let command_path = Path::new(command);
    let command_target = if command_path.has_root() {
        command_path.components().skip(1).collect::<PathBuf>()
    } else {
        command_path.to_path_buf()
    };

    fs::create_dir_all(tmp.join(&command_target).parent().unwrap()).unwrap();
    fs::copy(command, tmp.join(&command_target)).unwrap();

    unsafe {
        libc::chroot(cpath.as_ptr());
        libc::unshare(libc::CLONE_NEWPID);
    }

    env::set_current_dir("/").unwrap();

    let output = Command::new(command).args(command_args).output().unwrap();
    let std_out = std::str::from_utf8(&output.stdout).unwrap();
    let std_err = std::str::from_utf8(&output.stderr).unwrap();

    eprint!("{}", std_err);
    print!("{}", std_out);
    std::process::exit(output.status.code().unwrap());
}

fn get_image_blobs(name: &str, reference: &str) -> Result<Vec<Bytes>, String> {
    let headers = docker_auth_headers(name)?;
    let client = Client::new();
    let manifest_url = format!("https://registry.hub.docker.com/v2/library/{}/manifests/{}", name, reference);
    let manifest: Value = client
        .get(&manifest_url)
        .headers(headers.clone())
        .send()
        .unwrap()
        .json()
        .unwrap();
    let layers: Vec<&str> = manifest["fsLayers"]
        .as_array()
        .ok_or("Malformed manifest")?
        .iter()
        .map(|layer| layer["blobSum"].as_str().unwrap())
        .collect();
    let mut res = Vec::with_capacity(layers.len());

    for layer in layers {
        let layer_url = format!("https://registry.hub.docker.com/v2/library/{}/blobs/{}", name, layer);
        let layer_res = client.get(&layer_url).headers(headers.clone()).send().unwrap();

        let bytes = layer_res.bytes().unwrap();
        res.push(bytes);
    }

    Ok(res)
}

fn docker_auth_headers(image: &str) -> Result<HeaderMap, String> {
    let realm = "https://auth.docker.io/token";
    let service = "registry.docker.io";
    let scope = format!("repository:library/{}:pull", image);
    let token_url = format!("{}?service=registry.docker.io&scope={}", realm, scope);
    let token_res: Value = Client::new().get(&token_url).send().unwrap().json().unwrap();
    let token = token_res["token"].as_str().ok_or("No token in response")?;
    let www_authenticate = format!("Bearer realm=\"{}\",service=\"{}\",scope=\"{}\"", realm, service, scope);
    let authorization = format!("Bearer {}", token);

    let mut headers = HeaderMap::new();
    headers.insert(WWW_AUTHENTICATE, HeaderValue::from_str(&www_authenticate).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&authorization).unwrap());
    Ok(headers)
}

fn untar(bytes: &[u8], directory: &str) {
    let tar = Command::new("tar")
        .args(&["xfz", "-", "--directory", directory])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut tar_input = tar.stdin.unwrap();
    let mut writer = BufWriter::new(&mut tar_input);
    writer.write_all(bytes).unwrap();
}

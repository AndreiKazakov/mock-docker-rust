use std::env;
use std::ffi::CString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use bytes::Bytes;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, WWW_AUTHENTICATE};
use serde_json::Value;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let image_reference: Vec<&str> = args[2].split(':').collect();
    let image = image_reference[0];
    let tag = if image_reference.len() > 1 {
        image_reference[1]
    } else {
        "latest"
    };
    let command = Path::new(&args[3]);
    let command_args = &args[4..];
    let tmp = PathBuf::from("/app/sandbox");
    let cpath = CString::new("/app/sandbox").unwrap();

    fs::create_dir_all(tmp.join("dev")).unwrap();
    let dev_null = tmp.join("dev").join("null");
    Command::new("mknod")
        .args(&["-m", "666", dev_null.to_str().unwrap(), "c", "1", "3"])
        .status()
        .unwrap();

    let blobs = get_image_blobs(image, tag)?;

    for bytes in blobs {
        untar(&bytes, tmp.to_str().unwrap())?;
    }

    unsafe {
        libc::chroot(cpath.into_raw() as *const libc::c_char);
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
    let docker_hub = "https://registry.hub.docker.com";
    let manifest_url = format!("{}/v2/library/{}/manifests/{}", docker_hub, name, reference);
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
        let layer_url = format!("{}/v2/library/{}/blobs/{}", docker_hub, name, layer);
        let layer_res = client
            .get(&layer_url)
            .headers(headers.clone())
            .send()
            .unwrap();

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
    let token_res: Value = Client::new()
        .get(&token_url)
        .send()
        .unwrap()
        .json()
        .unwrap();
    let token = token_res["token"].as_str().ok_or("No token in response")?;
    let www_auth = format!(
        "Bearer realm=\"{}\",service=\"{}\",scope=\"{}\"",
        realm, service, scope
    );
    let auth = format!("Bearer {}", token);

    let mut headers = HeaderMap::new();
    headers.insert(WWW_AUTHENTICATE, HeaderValue::from_str(&www_auth).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth).unwrap());
    Ok(headers)
}

fn untar(bytes: &[u8], directory: &str) -> Result<(), String> {
    let mut tar = Command::new("tar")
        .args(&["xfz", "-", "--directory", directory])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    tar.stdin.take().unwrap().write_all(bytes).unwrap();
    match tar.wait() {
        Err(err) => Err(err.to_string()),
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(status.to_string()),
    }
}

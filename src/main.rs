use std::env;
use std::ffi::CString;
use std::fs;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let mut tmp = env::temp_dir();
    tmp.push("docker-rust-root");
    let cpath = CString::new(tmp.to_str().unwrap()).unwrap();
    fs::create_dir(&tmp).unwrap();
    fs::copy(command, tmp.join(command)).unwrap();

    unsafe {
        libc::chroot(cpath.as_ptr());
    }

    let output = std::process::Command::new(command)
        .args(command_args)
        .stdin(std::process::Stdio::piped())
        .output()
        .unwrap();
    let std_out = std::str::from_utf8(&output.stdout).unwrap();
    let std_err = std::str::from_utf8(&output.stderr).unwrap();

    eprint!("{}", std_err);
    print!("{}", std_out);
    fs::remove_dir_all(&tmp).unwrap();
    std::process::exit(output.status.code().unwrap());
}

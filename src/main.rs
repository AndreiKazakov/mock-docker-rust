use std::env::temp_dir;
use std::ffi::CString;
use std::fs;

fn main() {
    // Uncomment this block to pass the first stage!
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .unwrap();
    let tmp = temp_dir();
    let cpath = CString::new(tmp.to_str().unwrap()).unwrap();
    fs::copy(command, tmp.join(command)).unwrap();

    unsafe {
        libc::chroot(cpath.as_ptr());
    }

    let std_out = std::str::from_utf8(&output.stdout).unwrap();
    let std_err = std::str::from_utf8(&output.stderr).unwrap();

    eprint!("{}", std_err);
    print!("{}", std_out);
    std::process::exit(output.status.code().unwrap());
}

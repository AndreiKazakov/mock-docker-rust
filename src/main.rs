// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() {
    // Uncomment this block to pass the first stage!
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .unwrap();

    if output.status.success() {
        let std_out = std::str::from_utf8(&output.stdout).unwrap();
        let std_err = std::str::from_utf8(&output.stderr).unwrap();

        eprint!("{}", std_err);
        print!("{}", std_out);
        std::process::exit(output.status.code().unwrap());
    } else {
        std::process::exit(1);
    }
}

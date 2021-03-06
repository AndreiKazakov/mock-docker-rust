This is a starting point for Rust solutions to the
["Build Your Own Docker" Challenge](https://codecrafters.io/challenges/docker).

In this challenge, you'll build a program that can pull an image from
[Docker Hub](https://hub.docker.com/) and execute commands in it. Along the way,
we'll learn about [chroot](https://en.wikipedia.org/wiki/Chroot),
[kernel namespaces](https://en.wikipedia.org/wiki/Linux_namespaces), the
[docker registry API](https://docs.docker.com/registry/spec/api/) and much more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to signup for early access.

# Usage

1. Ensure you have [Docker](https://www.docker.com/) installed locally.
1. Follow the details below ("Running your program locally") to run your Docker
   implementation, which is implemented in `src/main.rs`.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.

# Running your program locally

Since you'll need to use linux-specific syscalls in this challenge, we'll run
your code _inside_ a docker container.

```sh
docker build -t my_docker . && docker run --cap-add="SYS_ADMIN" my_docker run some_image /usr/local/bin/docker-explorer echo hey
```

(The `--cap-add="SYS_ADMIN"` flag is required to create
[PID Namespaces](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html))

To make this easier to type out, you could add a
[shell alias](https://shapeshed.com/unix-alias/):

```sh
alias mydocker='docker build -t mydocker . && docker run --cap-add="SYS_ADMIN" mydocker'
```

You can then execute your program like this:

```sh
mydocker run ubuntu:latest /usr/local/bin/docker-explorer echo hey
```

This command compiles your Rust project, so it might be slow the first time you
run it. Subsequent runs will be fast.

# Passing the first stage

CodeCrafters runs tests when you do a `git push`. Make an empty commit and push
your solution to see the first stage fail.

```sh
git commit --allow-empty -m "Running tests"
git push origin master
```

Go to `src/main.rs` and uncomment the implementation. Commit and push your
changes to pass the first stage:

```sh
git add .
git commit -m "pass the first stage"
git push origin master
```

Time to move on to the next stage!

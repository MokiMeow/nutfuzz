#[cfg(target_os = "linux")]
fn main() {
    if let Err(error) = run_server() {
        eprintln!("nutfuzz forkserver target: {error}");
        std::process::exit(2);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("nutfuzz forkserver target is supported only on Linux");
    std::process::exit(2);
}

#[cfg(target_os = "linux")]
fn run_server() -> std::io::Result<()> {
    use std::io;

    use nutfuzz::protocol::{read_hello, read_run, write_hello};

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();
    let mut output = stdout.lock();

    read_hello(&mut input)?;
    write_hello(&mut output)?;
    while let Ok(candidate) = read_run(&mut input) {
        let reply = run_worker(&candidate);
        nutfuzz::protocol::write_reply(&mut output, &reply)?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_worker(input: &[u8]) -> nutfuzz::protocol::ForkserverReply {
    use std::fs::File;
    use std::os::fd::FromRawFd;

    use nutfuzz::exec::RunResult;
    use nutfuzz::protocol::{read_reply, write_reply, ForkserverReply};

    let mut descriptors = [0i32; 2];
    if unsafe { pipe(descriptors.as_mut_ptr()) } != 0 {
        return error_reply(std::io::Error::last_os_error());
    }
    let pid = unsafe { fork() };
    if pid < 0 {
        close_fd(descriptors[0]);
        close_fd(descriptors[1]);
        return error_reply(std::io::Error::last_os_error());
    }
    if pid == 0 {
        close_fd(descriptors[0]);
        if input == b"HANG" {
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        let mut coverage = nutfuzz::Coverage::new();
        let outcome = nutfuzz::target::parse_nested(input, &mut coverage);
        let result = match outcome {
            nutfuzz::Outcome::Ok => RunResult::Exited(0),
            nutfuzz::Outcome::Crash(message) => {
                RunResult::Crashed(11, message)
            }
        };
        let reply = ForkserverReply {
            result,
            coverage: coverage.snapshot(),
        };
        let mut pipe = unsafe { File::from_raw_fd(descriptors[1]) };
        let status = if write_reply(&mut pipe, &reply).is_ok() {
            0
        } else {
            2
        };
        unsafe { child_exit(status) };
    }

    close_fd(descriptors[1]);
    let mut pipe = unsafe { File::from_raw_fd(descriptors[0]) };
    let reply = read_reply(&mut pipe);
    let mut status = 0i32;
    unsafe {
        let _ = waitpid(pid, &mut status, 0);
    }
    reply.unwrap_or_else(error_reply)
}

#[cfg(target_os = "linux")]
fn error_reply(error: impl std::fmt::Display) -> nutfuzz::protocol::ForkserverReply {
    nutfuzz::protocol::ForkserverReply {
        result: nutfuzz::exec::RunResult::HarnessError(error.to_string()),
        coverage: Vec::new(),
    }
}

#[cfg(target_os = "linux")]
fn close_fd(descriptor: i32) {
    unsafe {
        let _ = close(descriptor);
    }
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn pipe(descriptors: *mut i32) -> i32;
    fn fork() -> i32;
    fn close(descriptor: i32) -> i32;
    fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    #[link_name = "_exit"]
    fn child_exit(status: i32) -> !;
}

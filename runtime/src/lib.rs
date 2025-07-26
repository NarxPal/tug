use nix::sched::{clone, CloneFlags};
use nix::sys::wait::waitpid;
use nix::unistd::{execv, getpid};
use std::ffi::CString;

const STACK_SIZE: usize = 1024 * 1024;

fn main() {
    let mut stack = vec![0u8; STACK_SIZE];

    let cb = Box::new(|| {
        println!("Inside child process: PID = {}", getpid());
        let path = CString::new("/bin/bash").unwrap();
        execv(&path, &[path.clone()]).unwrap();
        0
    });

    let child_pid = unsafe {
        clone(
            cb,
            &mut stack,
            CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWPID
                | CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWNET
                | CloneFlags::CLONE_NEWIPC,
            Some(nix::sys::signal::Signal::SIGCHLD as i32),
        )
    }
    .expect("clone failed");

    println!("Child PID: {}", child_pid);
    waitpid(child_pid, None).unwrap();
}

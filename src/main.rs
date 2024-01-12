//! This is a conceptually simple example that spawns the `whoami` program
//! to print your username.  It is made more complex because there are multiple
//! pipes involved and it is easy to get blocked/deadlocked if care and attention
//! is not paid to those pipes!
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::{
    io::{BufReader, Read, Write},
    sync::mpsc::channel,
};

fn main() {
    let pty_system = NativePtySystem::default();

    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let cmd = CommandBuilder::new("whoami");
    let mut child = pair.slave.spawn_command(cmd).unwrap();

    // Release any handles owned by the slave: we don't need it now
    // that we've spawned the child.
    drop(pair.slave);

    // Read the output in another thread.
    // This is important because it is easy to encounter a situation
    // where read/write buffers fill and block either your process
    // or the spawned process.
    let reader = pair.master.try_clone_reader().unwrap();
    let mut stdout = std::io::stdout();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        // stream output from pty to stdout
        std::io::copy(&mut reader, &mut stdout).unwrap();
    });

    // Wait for the child to complete
    println!("child status: {:?}", child.wait().unwrap());

    // Take care to drop the master after our processes are
    // done, as some platforms get unhappy if it is dropped
    // sooner than that.
    drop(pair.master);

    // Now wait for the output to be read by our reader thread
    // let output = rx.recv().unwrap();

    // // We print with escapes escaped because the windows conpty
    // // implementation synthesizes title change escape sequences
    // // in the output stream and it can be confusing to see those
    // // printed out raw in another terminal.
    // print!("output: ");
    // for c in output.escape_debug() {
    //     print!("{}", c);
    // }
}

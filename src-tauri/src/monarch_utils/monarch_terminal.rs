/*
* Monarch currently ships with a bundled Kitty instance.
* This makes it easier to run commandline programs that are
* needed, such as SteamCMD.
*
* TODO: Replace with a better future implementation that allows
* Monarch to more easily read the stdout and progress of commands
* run in terminal.
 */

use anyhow::{bail, Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};

/// Run a command in a new process and display to the user in a custom terminal window.
pub async fn run_in_terminal(command: &str) -> Result<()> {
    let pty_system = native_pty_system();

    let mut pair = pty_system.openpty(PtySize {
        rows: 60,
        cols: 160,
        // Not all systems support pixel_width, pixel_height,
        // but it is good practice to set it to something
        // that matches the size of the selected font.  That
        // is more complex than can be shown here in this
        // brief example though!
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Spawn a shell into the pty
    let mut cmd = CommandBuilder::new_default_prog();
    let shell = cmd.get_shell();

    cmd = CommandBuilder::new(shell);
    cmd.args(vec!["-c", command]);
    let child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| "Failed to spawn child commnad! | Err: ")?;

    // Read and parse output from the pty with reader
    let mut reader = pair.master.try_clone_reader()?;

    // Send data to the pty by writing to the master
    writeln!(pair.master.take_writer()?, "ls -l\r\n")?;

    Ok(())
}

use std::{process::Command, thread, time::Duration};
fn main() -> Result<(), anyhow::Error> {
    let s = sysinfo::System::new_all();

    let target_name = "test-app";
    let processes = s.processes_by_name(target_name);

    for process in processes {
        Command::new("kill")
            .arg(format!("{}", process.pid()))
            .output()?;
        thread::sleep(Duration::from_millis(1000));
    }

    let target_name = "http_server";
    let processes = s.processes_by_name(target_name);

    for process in processes {
        Command::new("kill")
            .arg(format!("{}", process.pid()))
            .output()?;
        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

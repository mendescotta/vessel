use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildEvent {
    Log(String),
    Finished(BuildResult),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildResult {
    Success,
    Failed(i32),
}

pub fn write_argfile(argv: &[String]) -> std::io::Result<PathBuf> {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("vessel-argv-{}-{unique}.txt", std::process::id()));
    let mut contents = argv.join("\n");
    contents.push('\n');
    std::fs::write(&path, contents)?;
    Ok(path)
}

pub fn spawn_build<F>(command: &str, args: &[String], on_event: F) -> thread::JoinHandle<()>
where
    F: Fn(BuildEvent) + Send + Sync + 'static,
{
    let command = command.to_string();
    let args = args.to_vec();
    thread::spawn(move || {
        let mut child = match Command::new(&command)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                on_event(BuildEvent::Finished(BuildResult::Failed(-1)));
                return;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                on_event(BuildEvent::Log(line));
            }
        }

        let result = match child.wait() {
            Ok(status) if status.success() => BuildResult::Success,
            Ok(status) => BuildResult::Failed(status.code().unwrap_or(-1)),
            Err(_) => BuildResult::Failed(-1),
        };
        on_event(BuildEvent::Finished(result));
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn streams_lines_and_reports_success() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let handle = spawn_build(
            "sh",
            &["-c".to_string(), "printf 'a\\nb\\n'".to_string()],
            move |event| events_clone.lock().unwrap().push(event),
        );
        handle.join().unwrap();
        let events = events.lock().unwrap().clone();
        assert_eq!(
            events,
            vec![
                BuildEvent::Log("a".to_string()),
                BuildEvent::Log("b".to_string()),
                BuildEvent::Finished(BuildResult::Success),
            ]
        );
    }

    #[test]
    fn reports_failure_exit_code() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let handle = spawn_build(
            "sh",
            &["-c".to_string(), "exit 3".to_string()],
            move |event| events_clone.lock().unwrap().push(event),
        );
        handle.join().unwrap();
        let events = events.lock().unwrap().clone();
        assert_eq!(events, vec![BuildEvent::Finished(BuildResult::Failed(3))]);
    }

    #[test]
    fn writes_argfile_one_arg_per_line_with_trailing_newline() {
        let argv = vec!["-a".to_string(), "x86_64".to_string(), "-p".to_string(), "firefox gimp".to_string()];
        let path = write_argfile(&argv).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(contents, "-a\nx86_64\n-p\nfirefox gimp\n");
    }
}

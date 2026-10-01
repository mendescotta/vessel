use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

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
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                on_event(BuildEvent::Finished(BuildResult::Failed(-1)));
                return;
            }
        };

        let on_event = Arc::new(on_event);
        let stderr_reader = child.stderr.take().map(|stderr| {
            let on_event = on_event.clone();
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    on_event(BuildEvent::Log(line));
                }
            })
        });
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                on_event(BuildEvent::Log(line));
            }
        }
        if let Some(reader) = stderr_reader {
            let _ = reader.join();
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
    use std::sync::Mutex;

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
    fn streams_stderr_lines_too() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let handle = spawn_build(
            "sh",
            &["-c".to_string(), "echo oops >&2; exit 1".to_string()],
            move |event| events_clone.lock().unwrap().push(event),
        );
        handle.join().unwrap();
        let events = events.lock().unwrap().clone();
        assert_eq!(
            events,
            vec![BuildEvent::Log("oops".to_string()), BuildEvent::Finished(BuildResult::Failed(1))]
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
}

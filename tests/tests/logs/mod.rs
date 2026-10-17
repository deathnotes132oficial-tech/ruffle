use ruffle_test_framework::environment::Environment;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use tracing::Level;
use tracing::field::Field;
use tracing_subscriber::layer::{Context, SubscriberExt};

struct Line {
    level: Level,
    message: String,
}

#[derive(Clone, Default)]
struct Log(Arc<Mutex<Vec<Line>>>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Log {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut message = String::new();
        event.record(&mut |_: &Field, value: &dyn Debug| message.push_str(&format!("{value:?}")));
        self.0.lock().unwrap().push(Line {
            level: *event.metadata().level(),
            message,
        });
    }
}

fn run_logged(
    environment: &impl Environment,
    tests: &[&str],
) -> Result<Vec<Line>, libtest_mimic::Failed> {
    let log = Log::default();
    let subscriber = tracing_subscriber::registry().with(log.clone());
    tracing::subscriber::with_default(subscriber, || -> Result<(), libtest_mimic::Failed> {
        for &name in tests {
            let dir = VfsPath::new(PhysicalFS::new(format!("tests/swfs/avm2/{name}/")));
            let options = TestOptions::read(&dir.join("test.toml")?)?;
            let test = &Test::from_options(options, dir, name.to_string())?;
            let mut runner = test.create_test_runner(environment)?;
            loop {
                match runner.tick()? {
                    TestStatus::Continue => {}
                    TestStatus::Sleep(duration) => sleep(duration),
                    TestStatus::Finished => break,
                }
            }
        }
        Ok(())
    })?;
    Ok(std::mem::take(&mut *log.0.lock().unwrap()))
}

pub fn loader_cancellations_are_not_errors(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    let lines = run_logged(
        environment,
        &["loader_cancel_in_handlers", "loader_reuse_and_close"],
    )?;
    let cancelled: Vec<_> = lines
        .iter()
        .filter(|line| line.level == Level::ERROR && line.message.contains("ancelled"))
        .map(|line| &line.message)
        .collect();
    if !cancelled.is_empty() {
        return Err(format!(
            "{} cancelled loads logged as errors: {cancelled:#?}",
            cancelled.len()
        )
        .into());
    }
    Ok(())
}

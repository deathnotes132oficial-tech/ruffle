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
    target: String,
    message: String,
    movie: String,
}

#[derive(Clone, Default)]
struct Log(Arc<Mutex<Vec<Line>>>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Log {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let (mut message, mut movie) = (String::new(), String::new());
        event.record(&mut |field: &Field, value: &dyn Debug| match field.name() {
            "movie" => movie = format!("{value:?}"),
            _ => message.push_str(&format!("{value:?}")),
        });
        self.0.lock().unwrap().push(Line {
            level: *event.metadata().level(),
            target: event.metadata().target().to_owned(),
            message,
            movie,
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

const UNCAUGHT_IN_CHAIN_TEST: [&str; 1] = ["boom1"];
const CANCELLED_IN_CHAIN_TEST: [&str; 3] = ["boom2", "boom3", "boom4"];
/// The movie whose code threw each cancelled error. boom2 and boom3 come from
/// anonymous closures, whose stacks name no class.
const CANCELLED_IN_CHAIN_TEST_FROM: [(&str, &str); 3] = [
    ("boom2", "child.swf"),
    ("boom3", "test.swf"),
    ("boom4", "child.swf"),
];

pub fn cancelled_uncaught_errors_are_logged(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    let lines = run_logged(environment, &["uncaught_error_events_chain"])?;
    let booms = |target: &str| -> Vec<&str> {
        let mut booms: Vec<_> = lines
            .iter()
            .filter(|line| line.target == target)
            .flat_map(|line| {
                UNCAUGHT_IN_CHAIN_TEST
                    .into_iter()
                    .chain(CANCELLED_IN_CHAIN_TEST)
                    .filter(|boom| line.message.contains(boom))
            })
            .collect();
        booms.sort();
        booms
    };
    let uncaught = booms("ruffle_core::avm2");
    let cancelled = booms("ruffle_core::avm2::uncaught_cancelled");
    if uncaught != UNCAUGHT_IN_CHAIN_TEST || cancelled != CANCELLED_IN_CHAIN_TEST {
        return Err(format!(
            "expected {UNCAUGHT_IN_CHAIN_TEST:?} logged as uncaught and {CANCELLED_IN_CHAIN_TEST:?} as cancelled, \
             got {uncaught:?} and {cancelled:?}"
        )
        .into());
    }
    for (boom, from) in CANCELLED_IN_CHAIN_TEST_FROM {
        let movie = lines
            .iter()
            .find(|line| {
                line.target == "ruffle_core::avm2::uncaught_cancelled"
                    && line.message.contains(boom)
            })
            .map(|line| line.movie.as_str());
        if !movie.is_some_and(|movie| movie.ends_with(&format!("/{from}\""))) {
            return Err(
                format!("{boom} should be logged as thrown by {from}, got {movie:?}").into(),
            );
        }
    }
    Ok(())
}

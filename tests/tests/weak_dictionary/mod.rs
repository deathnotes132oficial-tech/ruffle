//! `Dictionary(weakKeys = true)`: an entry goes away with its key.

use ruffle_test_framework::environment::Environment;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::thread::sleep;

/// Runs `avm2/dictionary_weak_keys` with a full collection after every frame, and expects
/// Flash's output (`output.full_gc.txt`, the same as `output.txt`): once the movie drops three
/// of its five keys, the dictionary counts two.
///
/// The test itself stays a known failure: it relies on Flash collecting within a few frames,
/// and Ruffle's paced collector doesn't get to it in time.
pub fn weak_keys_are_collected(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    let test = &Test::from_options(
        TestOptions {
            num_frames: Some(25),
            output_path: "output.full_gc.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new("tests/swfs/avm2/dictionary_weak_keys/")),
        "weak_keys_are_collected".to_string(),
    )?;

    let mut runner = test.create_test_runner(environment)?;
    loop {
        let status = runner.tick()?;
        runner.player().lock().unwrap().collect_all_garbage();
        match status {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }
    Ok(())
}

use ruffle_core::memory_report::MemoryReport;
use ruffle_test_framework::environment::Environment;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::thread::sleep;

const FRAME_BEFORE_PING: usize = 20;
const WEAK_LISTENERS: usize = 5000;
const MIN_FREED_BY_PING: usize = WEAK_LISTENERS * 4 / 5;

pub fn dead_weak_listeners_are_pruned_on_dispatch(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    let test = &Test::from_options(
        TestOptions {
            num_frames: Some(45),
            output_path: "output.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new(
            "tests/swfs/avm2/event_dispatcher_weak_listeners_pruned/",
        )),
        "event_dispatcher_weak_listeners_pruned".to_string(),
    )?;

    let mut runner = test.create_test_runner(environment)?;
    let mut objects = Vec::new();
    loop {
        let status = runner.tick()?;
        let mut player = runner.player().lock().unwrap();
        player.collect_all_garbage();
        objects.push(
            player.mutate_with_update_context(|context| MemoryReport::capture(context).gc_objects),
        );
        drop(player);
        match status {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    let before = objects[FRAME_BEFORE_PING];
    let after = *objects.last().unwrap();
    if before < after + MIN_FREED_BY_PING {
        return Err(format!(
            "the dispatch freed {} allocations, expected the {WEAK_LISTENERS} dead weak listeners' \
             ({before} before, {after} after)",
            before.saturating_sub(after)
        )
        .into());
    }
    Ok(())
}

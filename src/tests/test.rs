use crate::extractor::runner::collect_program_info;
use std::path::Path;

// #[test]
// fn gather_minimal() {
//     let cmd = gather_command("../target/debug/test-minimal").unwrap();
//     dbg!(&cmd);
//     assert!(false);
// }

#[test]
fn gather_test1() {
    let cmd = collect_program_info(Path::new("../target/debug/test-test1")).unwrap();
    dbg!(&cmd);
    assert!(false);
}

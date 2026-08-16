use std::process::Command;

#[test]
fn main() {
    let dir = env!("CARGO_MANIFEST_DIR");

    let tests = std::fs::read_dir(format!("{dir}/test_crates")).unwrap();

    for test in tests {
        let test = test.unwrap();
        let status = Command::new("./check.sh")
            .current_dir(test.path())
            .status()
            .unwrap();
        assert!(status.success());
    }
}

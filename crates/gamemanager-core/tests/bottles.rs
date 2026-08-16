use std::io;

use gamemanager_core::{BottlesCli, BottlesCommandOutput, BottlesCommandRunner, Result};

struct FakeRunner(Vec<BottlesCommandOutput>);

impl BottlesCommandRunner for FakeRunner {
    fn run(&self, _cli: &BottlesCli, args: &[&str]) -> io::Result<BottlesCommandOutput> {
        let index = usize::from(args != ["--json", "list", "bottles"]);
        Ok(self.0[index].clone())
    }
}

#[test]
fn json_object_form_preserves_discovered_bottle_names() -> Result<()> {
    let cli = BottlesCli::new("bottles-cli");
    let runner = FakeRunner(vec![BottlesCommandOutput::success(
        r#"{"bottles":[{"name":"Games"},{"Bottle":"Testing"}]}"#,
    )]);

    assert_eq!(cli.list_bottles_with(&runner)?, ["Games", "Testing"]);
    Ok(())
}

#[test]
fn json_array_form_preserves_discovered_bottle_names() -> Result<()> {
    let cli = BottlesCli::new("bottles-cli");
    let runner = FakeRunner(vec![BottlesCommandOutput::success(
        r#"["Games", {"id":"Testing"}, "Games"]"#,
    )]);

    assert_eq!(cli.list_bottles_with(&runner)?, ["Games", "Testing"]);
    Ok(())
}

#[test]
fn failed_json_falls_back_to_bulleted_text_without_invoking_bottles() -> Result<()> {
    let cli = BottlesCli::new("bottles-cli");
    let runner = FakeRunner(vec![
        BottlesCommandOutput::failure("unsupported"),
        BottlesCommandOutput::success("INFO scanning\n- Games\n* Testing\n"),
    ]);

    assert_eq!(cli.list_bottles_with(&runner)?, ["Games", "Testing"]);
    Ok(())
}

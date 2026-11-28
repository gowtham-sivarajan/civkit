// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-28 14:10 IST · last edited 2026-11-28 14:10 IST

//! The disclaimer (docs/ENGINEERING_STANDARDS.md §7) is one string in the engine. Python and MCP show this string;
//! the JSON contract does not carry it (D-13 parity is unchanged).

#[test]
fn the_disclaimer_is_the_one_in_the_engineering_standards() {
    let standards = include_str!("../../../docs/ENGINEERING_STANDARDS.md");
    let section = standards
        .split("## 7. Disclaimer")
        .nth(1)
        .expect("ENGINEERING_STANDARDS has §7");
    let quoted = section
        .lines()
        .find_map(|l| l.strip_prefix("> "))
        .expect("§7 quotes the disclaimer");
    assert_eq!(civkit_core::DISCLAIMER, quoted);
}

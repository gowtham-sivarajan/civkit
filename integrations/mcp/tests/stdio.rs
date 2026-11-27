// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:24 IST · last edited 2026-11-27 22:53 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! ADR 0013 gates G1 and G3 over real stdio: the built `civkit-mcp` binary is spawned and spoken to as a
//! 2025-11-25 client (`initialize` handshake) and as a 2026-07-28 client (per-request `_meta`, `server/discover`).

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use civkit_core::Registry;
use serde_json::{Map, Value, json};

const TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Copy, Debug)]
enum Era {
    /// 2025-11-25: the `initialize` handshake.
    Handshake,
    /// 2026-07-28: no handshake; every request carries its version in `_meta`.
    PerRequest,
}

const ERAS: [Era; 2] = [Era::Handshake, Era::PerRequest];

struct Client {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<String>,
    era: Era,
    next_id: u64,
}

impl Client {
    fn start(era: Era, args: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_civkit-mcp"))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("start civkit-mcp");
        let stdout = child.stdout.take().expect("stdout");
        let (tx, lines) = channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        let stdin = child.stdin.take();
        let mut client = Self {
            child,
            stdin,
            lines,
            era,
            next_id: 0,
        };
        if let Era::Handshake = era {
            let init = client.request(
                "initialize",
                json!({
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": { "name": "civkit-stdio-test", "version": "0" }
                }),
            );
            assert_eq!(init["result"]["protocolVersion"], "2025-11-25", "{init}");
            assert_eq!(init["result"]["serverInfo"]["name"], "civkit-mcp", "{init}");
            client.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        }
        client
    }

    fn send(&mut self, message: &Value) {
        let stdin = self.stdin.as_mut().expect("stdin open");
        writeln!(stdin, "{message}").expect("write");
        stdin.flush().expect("flush");
    }

    /// Sends one request and returns its response, skipping notifications. Every line on stdout must be JSON-RPC.
    fn request(&mut self, method: &str, mut params: Value) -> Value {
        if let Era::PerRequest = self.era {
            params["_meta"] = json!({
                "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                "io.modelcontextprotocol/clientCapabilities": {},
                "io.modelcontextprotocol/clientInfo": { "name": "civkit-stdio-test", "version": "0" }
            });
        }
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let line = self
                .lines
                .recv_timeout(left)
                .unwrap_or_else(|_| panic!("no response to {method} within {TIMEOUT:?}"));
            let message: Value = serde_json::from_str(&line)
                .unwrap_or_else(|e| panic!("stdout carries only MCP messages; got {line:?}: {e}"));
            assert_eq!(message["jsonrpc"], "2.0", "{line}");
            if message.get("id") == Some(&json!(id)) {
                return message;
            }
        }
    }

    fn tools(&mut self) -> Vec<Value> {
        let r = self.request("tools/list", json!({}));
        assert!(r["result"]["nextCursor"].is_null(), "one page: {r}");
        r["result"]["tools"].as_array().expect("tools").clone()
    }

    /// `tools/call`; returns the tool result (not the JSON-RPC envelope).
    fn call(&mut self, tool: &str, arguments: Value) -> Value {
        let r = self.request(
            "tools/call",
            json!({ "name": tool, "arguments": arguments }),
        );
        assert!(r.get("error").is_none(), "{tool}: protocol error {r}");
        r["result"].clone()
    }

    /// Closes stdin and waits for the server to exit by itself.
    fn finish(mut self) {
        drop(self.stdin.take());
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait().expect("wait") {
                assert!(status.success(), "exit status {status}");
                return;
            }
            assert!(
                Instant::now() < deadline,
                "the server must exit when stdin closes"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn engine(request: &Value) -> Value {
    let out = Registry::with_builtins().call_json(&request.to_string());
    serde_json::from_str(&out).expect("call_json returns JSON")
}

fn builtin_ids() -> BTreeSet<String> {
    Registry::with_builtins()
        .list()
        .iter()
        .map(|d| d.id.clone())
        .collect()
}

const GENERIC: [&str; 3] = ["civkit_call", "civkit_describe", "civkit_list"];

/// G3: structuredContent is `call_json`'s output; the text block holds the same JSON; `isError` matches.
fn assert_round_trip(result: &Value, expected: &Value, what: &str) {
    let is_error = expected.get("error").is_some() || expected.get("errors").is_some();
    assert_eq!(result["isError"], json!(is_error), "{what}: {result}");
    assert_eq!(&result["structuredContent"], expected, "{what}");
    let text = result["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("{what}: text block {result}"));
    let parsed: Value = serde_json::from_str(text).expect("the text block is JSON");
    assert_eq!(&parsed, expected, "{what}: text block");
}

fn reference_cases() -> Vec<(PathBuf, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference");
    let mut cases = Vec::new();
    for dir in std::fs::read_dir(&root).expect("tests/reference") {
        let dir = dir.expect("entry").path();
        if !dir.is_dir()
            || dir
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("math.demo."))
        {
            continue; // the template is registered only in civkit-core's tests
        }
        for file in std::fs::read_dir(&dir).expect("case dir") {
            let path = file.expect("entry").path();
            if path.extension().is_some_and(|e| e == "json") {
                let case: Value =
                    serde_json::from_str(&std::fs::read_to_string(&path).expect("read"))
                        .expect("json");
                cases.push((path, case["request"].clone()));
            }
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}

/// The per-calculation tool's arguments for a contract request: its inputs, plus `output_units` when given.
fn per_calc_arguments(request: &Value) -> Value {
    let mut args: Map<String, Value> = request["inputs"].as_object().cloned().unwrap_or_default();
    if let Some(units) = request.get("output_units") {
        args.insert("output_units".into(), units.clone());
    }
    Value::Object(args)
}

/// What the per-calculation tool must send to the engine: the request with the descriptor's version.
fn as_per_calc_request(request: &Value) -> Value {
    let mut r = request.clone();
    let id = r["id"].as_str().expect("id").to_owned();
    r["version"] = json!(
        Registry::with_builtins()
            .describe(&id, None)
            .expect("known")
            .version
    );
    r
}

fn assert_single_types(schema: &Value, at: &str) {
    match schema {
        Value::Object(map) => {
            if let Some(t) = map.get("type") {
                assert!(t.is_string(), "{at}: `type` must be one string, not {t}");
            }
            for (key, value) in map {
                assert_single_types(value, &format!("{at}.{key}"));
            }
        }
        Value::Array(items) => {
            for (i, value) in items.iter().enumerate() {
                assert_single_types(value, &format!("{at}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn g1_both_eras_list_every_tool_with_annotations_and_schemas() {
    for era in ERAS {
        let mut c = Client::start(era, &[]);
        let tools = c.tools();
        let names: BTreeSet<String> = tools
            .iter()
            .map(|t| t["name"].as_str().expect("name").to_owned())
            .collect();
        let mut expected = builtin_ids();
        expected.extend(GENERIC.iter().map(|s| s.to_string()));
        assert_eq!(names, expected, "{era:?}");
        for t in &tools {
            let name = t["name"].as_str().expect("name");
            assert_eq!(t["annotations"]["readOnlyHint"], true, "{era:?} {name}");
            assert_eq!(t["annotations"]["openWorldHint"], false, "{era:?} {name}");
            assert_eq!(t["inputSchema"]["type"], "object", "{era:?} {name}");
            assert_eq!(t["outputSchema"]["type"], "object", "{era:?} {name}");
            assert!(t["outputSchema"]["oneOf"].is_array(), "{era:?} {name}");
            // Portability (found by MCP Inspector --strict): every `type` is one string, alternatives use anyOf,
            // because some clients map tool schemas onto a single-`type` dialect.
            assert_single_types(&t["inputSchema"], &format!("{name} inputSchema"));
            assert_single_types(&t["outputSchema"], &format!("{name} outputSchema"));
        }
        let manning = tools
            .iter()
            .find(|t| t["name"] == "fluids.open_channel.manning_trapezoidal")
            .expect("manning");
        let d = Registry::with_builtins();
        let d = d
            .describe("fluids.open_channel.manning_trapezoidal", None)
            .expect("known");
        assert_eq!(manning["title"], d.title.as_str());
        assert_eq!(manning["description"], d.summary.as_str());
        let required: BTreeSet<&str> = manning["inputSchema"]["required"]
            .as_array()
            .expect("required")
            .iter()
            .map(|v| v.as_str().expect("name"))
            .collect();
        assert_eq!(
            required,
            d.inputs
                .iter()
                .filter(|i| i.required)
                .map(|i| i.name.as_str())
                .collect()
        );
        // A dimensional input needs its unit (D-19); a dimensionless one does not have one.
        let props = &manning["inputSchema"]["properties"];
        assert_eq!(
            props["depth"]["required"],
            json!(["value", "unit"]),
            "{era:?}"
        );
        assert_eq!(props["side_slope"]["required"], json!(["value"]), "{era:?}");
        assert!(
            props["side_slope"]["properties"]["unit"].is_null(),
            "{era:?}"
        );
        let earthwork = tools
            .iter()
            .find(|t| t["name"] == "quantities.earthwork.average_end_area")
            .expect("earthwork");
        assert_eq!(
            earthwork["inputSchema"]["properties"]["method"]["properties"]["value"]["enum"],
            json!(["average_end_area", "pyramid_at_zero_end"])
        );
        // A conditional requirement (prose, D-23 §3.9) is prominent: in the tool description and leading the input's.
        let beam = tools
            .iter()
            .find(|t| t["name"] == "structural.beam.simply_supported")
            .expect("beam");
        let description = beam["description"].as_str().expect("description");
        assert!(
            description.contains(
                "Conditional requirements: point_load_position is required when point_load > 0."
            ),
            "{era:?}: {description}"
        );
        let position = beam["inputSchema"]["properties"]["point_load_position"]["description"]
            .as_str()
            .expect("description");
        assert!(
            position.starts_with("Required when point_load > 0. "),
            "{era:?}: {position}"
        );
        assert!(
            !manning["description"]
                .as_str()
                .expect("d")
                .contains("Conditional"),
            "{era:?}"
        );
        let traverse = tools
            .iter()
            .find(|t| t["name"] == "surveying.traverse.compass_rule")
            .expect("traverse");
        let distances = &traverse["inputSchema"]["properties"]["distances"]["properties"]["value"];
        assert_eq!(distances["type"], "array");
        assert_eq!(distances["maxItems"], json!(civkit_core::MAX_LIST_LEN));
        c.finish();
    }
}

#[test]
fn g1_per_request_era_answers_discover_with_both_versions() {
    let mut c = Client::start(Era::PerRequest, &[]);
    let r = c.request("server/discover", json!({}));
    let versions = &r["result"]["supportedVersions"];
    assert!(
        versions
            .as_array()
            .is_some_and(|v| v.contains(&json!("2026-07-28")) && v.contains(&json!("2025-11-25"))),
        "{r}"
    );
    assert!(r["result"]["capabilities"]["tools"].is_object(), "{r}");
    c.finish();
}

/// G1 (scalar, list and choice calculations) and G3 (round trip) for every reference case, in both eras, through
/// the per-calculation tool and through `civkit_call`.
#[test]
fn g1_g3_every_reference_case_round_trips_in_both_eras() {
    let cases = reference_cases();
    let ids: BTreeSet<&str> = cases
        .iter()
        .map(|(_, r)| r["id"].as_str().expect("id"))
        .collect();
    assert_eq!(
        ids.len(),
        builtin_ids().len(),
        "every built-in calculation has a reference case"
    );
    for era in ERAS {
        let mut c = Client::start(era, &[]);
        for (path, request) in &cases {
            let what = format!("{era:?} {}", path.display());
            let id = request["id"].as_str().expect("id");
            let via_tool = c.call(id, per_calc_arguments(request));
            assert_round_trip(&via_tool, &engine(&as_per_calc_request(request)), &what);
            let via_generic = c.call("civkit_call", request.clone());
            assert_round_trip(
                &via_generic,
                &engine(request),
                &format!("{what} (civkit_call)"),
            );
        }
        c.finish();
    }
}

#[test]
fn g3_contract_errors_keep_is_error_and_the_envelope() {
    let manning = json!({
        "bottom_width": { "value": 1.5, "unit": "m" }, "side_slope": { "value": 2.0 },
        "depth": { "value": 0.6, "unit": "m" }, "manning_n": { "value": 0.013 }, "bed_slope": { "value": 0.002 }
    });
    let mut missing_unit = manning.clone();
    missing_unit["depth"] = json!({ "value": 0.6 });
    let mut several = manning.clone();
    several["bottom_width"] = json!({ "value": -1, "unit": "m" });
    several["manning_n"] = json!({ "value": 0 });
    let list_element = json!({
        "bearings": { "value": [0, 90, 180], "unit": "deg" }, "distances": { "value": [10, -1, 10], "unit": "m" }
    });
    let bad_choice = json!({
        "method": { "value": "prismoidal" }, "chainages": { "value": [0, 50], "unit": "m" },
        "cut_areas": { "value": [0, 30], "unit": "m^2" }, "fill_areas": { "value": [0, 0], "unit": "m^2" }
    });
    let mut unknown_output = manning.clone();
    unknown_output["output_units"] = json!({ "nothing": "m" });
    let per_calc = [
        (
            "fluids.open_channel.manning_trapezoidal",
            missing_unit,
            "MISSING_UNIT",
        ),
        (
            "fluids.open_channel.manning_trapezoidal",
            several,
            "OUT_OF_DOMAIN",
        ),
        (
            "surveying.traverse.compass_rule",
            list_element,
            "OUT_OF_DOMAIN",
        ),
        (
            "quantities.earthwork.average_end_area",
            bad_choice,
            "INVALID_VALUE",
        ),
        (
            "fluids.open_channel.manning_trapezoidal",
            unknown_output,
            "UNKNOWN_OUTPUT",
        ),
    ];
    for era in ERAS {
        let mut c = Client::start(era, &[]);
        for (tool, args, code) in &per_calc {
            let mut request = json!({ "id": tool, "inputs": args });
            if let Some(units) = request["inputs"]
                .as_object_mut()
                .and_then(|m| m.remove("output_units"))
            {
                request["output_units"] = units;
            }
            let expected = engine(&as_per_calc_request(&request));
            let result = c.call(tool, args.clone());
            assert_round_trip(&result, &expected, &format!("{era:?} {tool} {code}"));
            let first = expected
                .get("error")
                .cloned()
                .unwrap_or_else(|| expected["errors"][0].clone());
            assert_eq!(first["code"], *code, "{era:?} {tool}: {expected}");
        }
        // The list-element error keeps its index.
        let r = c.call(per_calc[2].0, per_calc[2].1.clone());
        assert_eq!(r["structuredContent"]["error"]["index"], 1, "{era:?}: {r}");
        assert_eq!(
            r["structuredContent"]["error"]["input"], "distances",
            "{era:?}: {r}"
        );
        // Request-level errors through civkit_call.
        for request in [
            json!({ "id": "fluids.open_channel.manning_trapezoidal", "version": 99, "inputs": manning }),
            json!({ "id": "fluids.nothing.here", "inputs": {} }),
            json!({ "id": "fluids.open_channel.manning_trapezoidal" , "inputs": manning, "extra": 1 }),
        ] {
            let result = c.call("civkit_call", request.clone());
            assert_round_trip(
                &result,
                &engine(&request),
                &format!("{era:?} civkit_call {request}"),
            );
        }
        c.finish();
    }
}

#[test]
fn generic_list_and_describe() {
    for era in ERAS {
        let mut c = Client::start(era, &[]);
        let list = c.call("civkit_list", json!({}));
        assert_eq!(list["isError"], false);
        let listed: BTreeSet<String> = list["structuredContent"]["calculations"]
            .as_array()
            .expect("calculations")
            .iter()
            .map(|d| d["id"].as_str().expect("id").to_owned())
            .collect();
        assert_eq!(listed, builtin_ids(), "{era:?}");
        let registry = Registry::with_builtins();
        for id in builtin_ids() {
            let described = c.call("civkit_describe", json!({ "id": id }));
            let expected = serde_json::to_value(registry.describe(&id, None).expect("known"))
                .expect("descriptor");
            assert_round_trip(&described, &expected, &format!("{era:?} describe {id}"));
        }
        let unknown = c.call("civkit_describe", json!({ "id": "fluids.nothing.here" }));
        assert_eq!(unknown["isError"], true);
        assert_eq!(
            unknown["structuredContent"]["error"]["code"],
            "UNKNOWN_CALCULATION"
        );
        let unknown_version = c.call(
            "civkit_describe",
            json!({ "id": "fluids.open_channel.manning_trapezoidal", "version": 99 }),
        );
        assert_eq!(
            unknown_version["structuredContent"]["error"]["code"],
            "UNKNOWN_VERSION"
        );
        c.finish();
    }
}

#[test]
fn an_unknown_tool_is_a_protocol_error() {
    for era in ERAS {
        let mut c = Client::start(era, &[]);
        let r = c.request(
            "tools/call",
            json!({ "name": "fluids.nothing.here", "arguments": {} }),
        );
        assert!(r.get("result").is_none(), "{era:?}: {r}");
        assert_eq!(r["error"]["code"], -32602, "{era:?}: {r}");
        c.finish();
    }
}

#[test]
fn the_tools_option_selects_the_surface() {
    let names = |args: &[&str]| {
        let mut c = Client::start(Era::Handshake, args);
        let names: BTreeSet<String> = c
            .tools()
            .iter()
            .map(|t| t["name"].as_str().expect("name").to_owned())
            .collect();
        c.finish();
        names
    };
    assert_eq!(names(&["--tools", "per-calc"]), builtin_ids());
    assert_eq!(
        names(&["--tools", "generic"]),
        GENERIC
            .iter()
            .map(|s| s.to_string())
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        names(&["--tools", "both"]).len(),
        builtin_ids().len() + GENERIC.len()
    );
    // A per-calculation tool is not callable when only the generic ones are exposed.
    let mut c = Client::start(Era::Handshake, &["--tools", "generic"]);
    let r = c.request(
        "tools/call",
        json!({ "name": "fluids.open_channel.manning_trapezoidal", "arguments": {} }),
    );
    assert!(r.get("error").is_some(), "{r}");
    c.finish();
}

#[test]
fn a_bad_option_exits_with_an_error_and_writes_nothing_to_stdout() {
    let out = Command::new(env!("CARGO_BIN_EXE_civkit-mcp"))
        .args(["--tools", "some"])
        .stdin(Stdio::null())
        .output()
        .expect("run");
    assert!(!out.status.success());
    assert!(out.stdout.is_empty(), "stdout is for MCP messages only");
    assert!(String::from_utf8_lossy(&out.stderr).contains("--tools"));
}

// ------------------------------------------------------------------------------------- drawing.analyze

/// The drawing tool (D-28 / ADR 0016 Q-A, API_DESIGN §10 and §13; gate A8): present only with a
/// `--drawing-root`, read-only, and it reads only files whose real path (symlinks resolved) lies inside a
/// root.
const DRAWING: &str = "drawing.analyze";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/dxf")
}

const DXF_FIXTURES: [&str; 7] = [
    "f1_site_plan_m.dxf",
    "f2_site_plan_mm.dxf",
    "f3_units_mismatch.dxf",
    "f4_legacy_r12.dxf",
    "f5_blocks_inserts.dxf",
    "third_party/ezdxf/Minimal_DXF_AC1009.dxf",
    "third_party/ezdxf/insert_bricscad_level_1.dxf",
];

/// The Rust report for a file, parsed: what the tool must return.
fn rust_report(path: &Path, entity_rows: bool) -> Value {
    let options = civkit_drawing::AnalysisOptions {
        entity_rows,
        ..civkit_drawing::AnalysisOptions::default()
    };
    let bytes = std::fs::read(path).expect("read fixture");
    let a = civkit_drawing::analyze_dxf(&bytes, &options).expect("analyses");
    serde_json::from_str(&a.to_json().expect("json")).expect("valid JSON")
}

/// A fresh, empty directory for one test.
fn scratch(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!("civkit-mcp-{}-{nanos}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// A symbolic link to a file or directory. On Unix it must succeed (CI on Linux and macOS runs every case).
/// On Windows a file symlink needs a privilege the test may not have (`false`); a directory link falls back to a
/// junction, which needs none.
fn symlink(target: &Path, link: &Path, dir: bool) -> bool {
    #[cfg(unix)]
    {
        let _ = dir;
        std::os::unix::fs::symlink(target, link).expect("symlinks work on Unix");
        true
    }
    #[cfg(windows)]
    {
        if !dir {
            return std::os::windows::fs::symlink_file(target, link).is_ok();
        }
        std::os::windows::fs::symlink_dir(target, link).is_ok()
            || Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(link)
                .arg(target)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
    }
}

fn path_arg(p: &Path) -> Value {
    json!({ "path": p.to_str().expect("UTF-8 path") })
}

/// A refused call: `isError`, and the contract envelope with the code and input.
fn assert_refused(result: &Value, code: &str, input: Option<&str>, what: &str) {
    assert_eq!(result["isError"], true, "{what}: {result}");
    let e = &result["structuredContent"]["error"];
    assert_eq!(e["code"], code, "{what}: {result}");
    if let Some(input) = input {
        assert_eq!(e["input"], input, "{what}: {result}");
    }
}

#[test]
fn the_drawing_tool_exists_only_with_a_drawing_root() {
    let mut c = Client::start(Era::Handshake, &[]);
    assert!(c.tools().iter().all(|t| t["name"] != DRAWING));
    let r = c.request("tools/call", json!({ "name": DRAWING, "arguments": {} }));
    assert!(r.get("error").is_some(), "no tool without a root: {r}");
    c.finish();

    let root = fixtures();
    let mut c = Client::start(
        Era::Handshake,
        &["--drawing-root", root.to_str().expect("path")],
    );
    let tools = c.tools();
    let t = tools
        .iter()
        .find(|t| t["name"] == DRAWING)
        .expect("drawing tool");
    assert_eq!(t["annotations"]["readOnlyHint"], true);
    assert_eq!(t["annotations"]["openWorldHint"], false);
    assert_eq!(t["inputSchema"]["required"], json!(["path"]));
    // A relative path is refused even when, from the server's working directory (this package's
    // directory, which the test process and so the server start in), it names a file inside the root.
    let relative = "../../tests/fixtures/dxf/f1_site_plan_m.dxf";
    assert!(
        Path::new(relative).is_file(),
        "the relative path names a real fixture"
    );
    assert_refused(
        &c.call(DRAWING, json!({ "path": relative })),
        "INVALID_VALUE",
        Some("path"),
        "relative path to a file inside the root",
    );
    assert!(
        t["description"]
            .as_str()
            .is_some_and(|d| d.contains("data")),
        "drawing text is data: {t}"
    );
    c.finish();
}

/// Every fixture, both eras, both details: `structuredContent` and the text block equal the Rust report.
#[test]
fn the_drawing_tool_returns_the_rust_report_in_both_eras() {
    let root = std::fs::canonicalize(fixtures()).expect("fixtures");
    for era in ERAS {
        let mut c = Client::start(era, &["--drawing-root", root.to_str().expect("path")]);
        for name in DXF_FIXTURES {
            let path = root.join(name);
            for (detail, entity_rows) in [("layer", false), ("entity", true)] {
                let mut args = path_arg(&path);
                args["detail"] = json!(detail);
                let r = c.call(DRAWING, args);
                let expected = rust_report(&path, entity_rows);
                assert!(
                    r.get("isError").is_none_or(|e| e == false),
                    "{era:?} {name}: {r}"
                );
                assert_eq!(r["structuredContent"], expected, "{era:?} {name} {detail}");
                let text: Value =
                    serde_json::from_str(r["content"][0]["text"].as_str().expect("text"))
                        .expect("text block is JSON");
                assert_eq!(text, expected, "{era:?} {name} {detail}: text block");
            }
        }
        c.finish();
    }
}

/// Only files whose real path is inside a root are read; everything else is `INVALID_VALUE` on `path`.
#[test]
fn paths_outside_the_roots_are_refused_including_through_symlinks() {
    let f1 = fixtures().join("f1_site_plan_m.dxf");
    let root = std::fs::canonicalize(scratch("root")).expect("root");
    let outside = std::fs::canonicalize(scratch("outside")).expect("outside");
    std::fs::copy(&f1, root.join("f1.dxf")).expect("copy");
    std::fs::copy(&f1, outside.join("f1.dxf")).expect("copy");
    std::fs::write(
        root.join("broken.dxf"),
        b"0\nSECTION\n2\nENTITIES\n0\nLINE\n10\nx\n",
    )
    .expect("write");
    std::fs::create_dir(root.join("folder")).expect("dir");

    let mut c = Client::start(
        Era::Handshake,
        &["--drawing-root", root.to_str().expect("path")],
    );
    let ok = c.call(DRAWING, path_arg(&root.join("f1.dxf")));
    assert_eq!(
        ok["structuredContent"],
        rust_report(&f1, false),
        "inside the root"
    );

    let climb = root
        .join("..")
        .join(outside.file_name().expect("name"))
        .join("f1.dxf");
    for (what, path) in [
        ("outside the root", outside.join("f1.dxf")),
        ("climbing out with ..", climb),
        ("missing", root.join("missing.dxf")),
        ("a directory", root.join("folder")),
    ] {
        assert_refused(
            &c.call(DRAWING, path_arg(&path)),
            "INVALID_VALUE",
            Some("path"),
            what,
        );
    }
    assert_refused(
        &c.call(DRAWING, json!({ "path": "f1.dxf" })),
        "INVALID_VALUE",
        Some("path"),
        "relative",
    );
    assert_refused(
        &c.call(DRAWING, json!({})),
        "MISSING_INPUT",
        Some("path"),
        "no path",
    );
    assert_refused(
        &c.call(DRAWING, json!({ "path": 7 })),
        "INVALID_VALUE",
        Some("path"),
        "not a string",
    );
    let mut bad_detail = path_arg(&root.join("f1.dxf"));
    bad_detail["detail"] = json!("everything");
    assert_refused(
        &c.call(DRAWING, bad_detail),
        "INVALID_VALUE",
        Some("detail"),
        "detail",
    );
    assert_refused(
        &c.call(DRAWING, path_arg(&root.join("broken.dxf"))),
        "DRAWING_UNREADABLE",
        None,
        "unreadable",
    );

    // A directory link out of the root (a junction on Windows if need be): the file's real path is outside.
    let dir_link = root.join("away");
    assert!(
        symlink(&outside, &dir_link, true),
        "a directory link can be made"
    );
    assert_refused(
        &c.call(DRAWING, path_arg(&dir_link.join("f1.dxf"))),
        "INVALID_VALUE",
        Some("path"),
        "directory link out",
    );
    // File symlinks: one inside the root to a file outside (refused), and one to a file inside (allowed:
    // its real path is inside).
    let file_link = root.join("link.dxf");
    let inner_link = root.join("inner.dxf");
    if symlink(&outside.join("f1.dxf"), &file_link, false)
        && symlink(&root.join("f1.dxf"), &inner_link, false)
    {
        assert_refused(
            &c.call(DRAWING, path_arg(&file_link)),
            "INVALID_VALUE",
            Some("path"),
            "file symlink out",
        );
        let inner = c.call(DRAWING, path_arg(&inner_link));
        assert_eq!(
            inner["structuredContent"],
            rust_report(&f1, false),
            "symlink to a file inside"
        );
    } else {
        eprintln!(
            "file-symlink cases skipped: Windows without the symlink privilege (Unix CI runs them)"
        );
    }
    c.finish();
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn a_drawing_root_that_is_not_a_directory_is_a_startup_error() {
    let missing = scratch("gone").join("not-here");
    let out = Command::new(env!("CARGO_BIN_EXE_civkit-mcp"))
        .args(["--drawing-root", missing.to_str().expect("path")])
        .stdin(Stdio::null())
        .output()
        .expect("run");
    assert!(!out.status.success());
    assert!(out.stdout.is_empty(), "stdout is for MCP messages only");
    assert!(String::from_utf8_lossy(&out.stderr).contains("--drawing-root"));
}

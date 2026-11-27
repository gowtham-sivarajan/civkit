// Provenance: Claude Sonnet 5.5 (claude-sonnet-5-5) · first drafted 2026-11-27 00:24 IST · last edited 2026-11-27 00:24 IST

//! The MCP handler. Every calculation goes through `Registry::call_json`; results map onto MCP as ADR 0013 point 4
//! fixes: success → `structuredContent` = the response, contract error → `isError: true` with the envelope, both
//! also as a text block. An unknown tool is a JSON-RPC protocol error.

use civkit_core::Registry;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::{Map, Value, json};

use crate::tools;

/// Which tools are exposed (`--tools`, API_DESIGN §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// One tool per calculation.
    PerCalc,
    /// `civkit_list`, `civkit_describe`, `civkit_call`.
    Generic,
    /// Both (the default for 0.1).
    Both,
}

impl Surface {
    /// Parses the `--tools` value.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "per-calc" => Some(Self::PerCalc),
            "generic" => Some(Self::Generic),
            "both" => Some(Self::Both),
            _ => None,
        }
    }

    fn per_calc(self) -> bool {
        self != Self::Generic
    }

    fn generic(self) -> bool {
        self != Self::PerCalc
    }
}

/// The CivKit MCP server.
pub struct CivKit {
    registry: Registry,
    surface: Surface,
    tools: Vec<Tool>,
}

impl CivKit {
    /// A server over the built-in calculations.
    pub fn new(surface: Surface) -> Self {
        let registry = Registry::with_builtins();
        let mut tools = Vec::new();
        if surface.per_calc() {
            tools.extend(registry.list().into_iter().map(tools::per_calculation));
        }
        if surface.generic() {
            let ids: Vec<String> = registry.list().iter().map(|d| d.id.clone()).collect();
            tools.extend(tools::generic(&ids));
        }
        Self {
            registry,
            surface,
            tools,
        }
    }

    /// Runs one contract request (as JSON) and maps the engine's answer onto a tool result.
    fn run(&self, request: &Value) -> CallToolResult {
        let answer = self.registry.call_json(&request.to_string());
        match serde_json::from_str::<Value>(&answer) {
            Ok(value) => result(value),
            Err(e) => CallToolResult::structured_error(envelope(
                "INTERNAL",
                &format!("the engine returned invalid JSON: {e}"),
                None,
            )),
        }
    }

    fn per_calc(&self, id: &str, arguments: Option<Map<String, Value>>) -> Option<CallToolResult> {
        let d = self.registry.describe(id, None)?;
        let mut inputs = arguments.unwrap_or_default();
        let mut request = json!({ "id": id, "version": d.version });
        if let Some(units) = inputs.remove("output_units") {
            request["output_units"] = units;
        }
        request["inputs"] = Value::Object(inputs);
        Some(self.run(&request))
    }

    fn list(&self) -> CallToolResult {
        let calculations: Vec<Value> = self
            .registry
            .list()
            .iter()
            .map(|d| json!({ "id": d.id, "version": d.version, "title": d.title, "summary": d.summary }))
            .collect();
        result(json!({ "calculations": calculations }))
    }

    fn describe(&self, arguments: &Map<String, Value>) -> CallToolResult {
        let Some(id) = arguments.get("id").and_then(Value::as_str) else {
            return CallToolResult::structured_error(envelope(
                "MISSING_INPUT",
                "civkit_describe needs \"id\", a calculation ID as a string",
                Some("id"),
            ));
        };
        let version = match arguments.get("version") {
            None | Some(Value::Null) => None,
            Some(v) => match v.as_u64().and_then(|v| u32::try_from(v).ok()) {
                Some(v) => Some(v),
                None => {
                    return CallToolResult::structured_error(envelope(
                        "INVALID_VALUE",
                        "\"version\" must be a positive integer",
                        Some("version"),
                    ));
                }
            },
        };
        match self.registry.describe(id, version) {
            Some(d) => match serde_json::to_value(d) {
                Ok(value) => result(value),
                Err(e) => {
                    CallToolResult::structured_error(envelope("INTERNAL", &e.to_string(), None))
                }
            },
            None if self.registry.describe(id, None).is_some() => {
                CallToolResult::structured_error(envelope(
                    "UNKNOWN_VERSION",
                    &format!("{id} has no version {}", version.unwrap_or_default()),
                    None,
                ))
            }
            None => CallToolResult::structured_error(envelope(
                "UNKNOWN_CALCULATION",
                &format!("no calculation has the ID {id}; civkit_list lists them"),
                None,
            )),
        }
    }
}

/// A contract envelope or response onto a tool result: `isError` exactly when it is an error envelope.
fn result(value: Value) -> CallToolResult {
    if value.get("error").is_some() || value.get("errors").is_some() {
        CallToolResult::structured_error(value)
    } else {
        CallToolResult::structured(value)
    }
}

/// A one-error envelope, for the generic tools' own argument errors (same shape as the engine's).
fn envelope(code: &str, message: &str, input: Option<&str>) -> Value {
    let mut error = json!({ "code": code, "message": message });
    if let Some(input) = input {
        error["input"] = json!(input);
    }
    json!({ "error": error })
}

impl ServerHandler for CivKit {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("civkit-mcp", civkit_core::VERSION))
            .with_instructions(
                "CivKit runs civil-engineering calculations offline and returns the full result: outputs, the \
                 calculation trace, warnings, references and assumptions. Report CivKit's numbers and explain them \
                 from the trace; do not recompute them. Every dimensional input needs a unit.",
            )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tools.clone()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let name = request.name.as_ref();
        let generic = self.surface.generic();
        let result = match name {
            tools::LIST if generic => self.list(),
            tools::DESCRIBE if generic => self.describe(&request.arguments.unwrap_or_default()),
            tools::CALL if generic => {
                self.run(&Value::Object(request.arguments.unwrap_or_default()))
            }
            id if self.surface.per_calc() => match self.per_calc(id, request.arguments) {
                Some(result) => result,
                None => return Err(unknown_tool(name)),
            },
            _ => return Err(unknown_tool(name)),
        };
        Ok(result.into())
    }
}

fn unknown_tool(name: &str) -> ErrorData {
    ErrorData::invalid_params(format!("unknown tool: {name}"), None)
}

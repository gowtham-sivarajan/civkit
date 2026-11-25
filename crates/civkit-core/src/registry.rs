// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-25 23:47 IST by Claude Sonnet 5.5 (claude-sonnet-5-5)

//! The calculation trait and the registry that validates requests and runs calculations.

use crate::contract::{
    InputValue, Level, Number, OutputValue, Request, Response, TraceItem, TraceStep, Warning,
    WarningCode,
};
use crate::descriptor::{Breach, Descriptor, InputKind, InputSpec, OutputKind};
use crate::error::{CalcError, ErrorCode, ErrorEnvelope};
use civkit_units::{Dimension, Unit, UnitError, bearing_to_azimuth, dms_to_radians};
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Domains allowed as the first segment of a calculation ID (`docs/API_DESIGN.md` §4).
pub const DOMAINS: &[&str] = &[
    "geotech",
    "structural",
    "surveying",
    "fluids",
    "quantities",
    "math",
    "units",
    "geometry",
    "drawing",
];

/// One calculation. Implement this once; every interface is generated from the descriptor.
///
/// The registry has already checked, before `compute` runs, that every required input is present, every
/// value is a finite number in SI, every value lies in its input's domain, and every validity rule holds
/// (or produced a warning). `compute` only does the engineering.
pub trait Calculation: Send + Sync {
    /// The calculation's self-description.
    fn descriptor(&self) -> &Descriptor;

    /// Computes the outputs, recording each step in `ctx`.
    fn compute(&self, inputs: &Inputs, ctx: &mut Context) -> Result<Outputs, CalcError>;
}

/// Most elements in any list, input or output (D-23). A group's maximum leaves room for its +1 outputs.
pub const MAX_LIST_LEN: usize = 10_000;

/// Validated input values, in SI, by name: scalars, lists and choices (D-23).
#[derive(Debug, Clone, Default)]
pub struct Inputs {
    scalars: BTreeMap<String, f64>,
    lists: BTreeMap<String, Vec<f64>>,
    choices: BTreeMap<String, String>,
}

impl Inputs {
    /// The value of a scalar input, if present (optional inputs without a default may be absent).
    pub fn get(&self, name: &str) -> Option<f64> {
        self.scalars.get(name).copied()
    }

    /// The value of a required scalar input. Returns `INTERNAL` if absent, which would be a CivKit bug,
    /// because the registry checks required inputs before `compute` runs.
    pub fn require(&self, name: &str) -> Result<f64, CalcError> {
        self.get(name).ok_or_else(|| {
            CalcError::internal(format!("required input '{name}' reached compute() missing"))
        })
    }

    /// The values of a list input, in SI. Lists are always required (D-23), so absence is `INTERNAL`.
    pub fn list(&self, name: &str) -> Result<&[f64], CalcError> {
        self.lists.get(name).map(Vec::as_slice).ok_or_else(|| {
            CalcError::internal(format!("list input '{name}' reached compute() missing"))
        })
    }

    /// The selected value of a choice input. Choices are always required (D-23), so absence is `INTERNAL`.
    pub fn choice(&self, name: &str) -> Result<&str, CalcError> {
        self.choices.get(name).map(String::as_str).ok_or_else(|| {
            CalcError::internal(format!("choice input '{name}' reached compute() missing"))
        })
    }
}

/// Output values, in SI, by name.
#[derive(Debug, Clone, Default)]
pub struct Outputs(BTreeMap<String, Number>);

impl Outputs {
    /// An empty set of outputs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a scalar output (builder style).
    #[must_use]
    pub fn with(mut self, name: &str, value: f64) -> Self {
        self.0.insert(name.to_owned(), Number::Scalar(value));
        self
    }

    /// Sets a list output (builder style). Its length must follow its declared index set (D-23).
    #[must_use]
    pub fn with_list(mut self, name: &str, values: Vec<f64>) -> Self {
        self.0.insert(name.to_owned(), Number::List(values));
        self
    }
}

/// Collects the trace and warnings while a calculation runs.
#[derive(Debug, Default)]
pub struct Context {
    trace: Vec<TraceStep>,
    warnings: Vec<Warning>,
}

impl Context {
    /// Records one numeric step of the trace.
    ///
    /// `unit` must be an internal unit: coherent SI without prefix (e.g. `m`, `Pa`, `m^3/s`), or `None` when
    /// dimensionless. The registry rejects a response whose trace uses any other unit (`INTERNAL`).
    pub fn step(&mut self, expression: &str, value: f64, unit: Option<&str>) {
        self.trace.push(TraceStep {
            step: expression.to_owned(),
            value: Some(value),
            unit: unit.map(str::to_owned),
            ..TraceStep::default()
        });
    }

    /// Records a numeric step about one entry of an index set (D-23). The text becomes
    /// `"<label> <index + 1>: <expression>"`, generated here so it always agrees with the structured `item`.
    pub fn step_item(
        &mut self,
        label: &str,
        index: usize,
        expression: &str,
        value: f64,
        unit: Option<&str>,
    ) {
        self.trace.push(TraceStep {
            step: format!("{label} {}: {expression}", index + 1),
            value: Some(value),
            unit: unit.map(str::to_owned),
            item: Some(TraceItem {
                label: label.to_owned(),
                index,
            }),
            ..TraceStep::default()
        });
    }

    /// Records the selected value of a choice input (D-23). Every choice input must be recorded exactly once,
    /// before any numeric step, in declaration order; the registry checks this (`INTERNAL` otherwise).
    pub fn choice(&mut self, input: &str, value: &str) {
        self.trace.push(TraceStep {
            step: input.to_owned(),
            choice: Some(value.to_owned()),
            ..TraceStep::default()
        });
    }

    /// Records a fully formed step. Low level: prefer [`Context::step`], [`Context::step_item`] and
    /// [`Context::choice`]. The registry checks every step after `compute`.
    pub fn push(&mut self, step: TraceStep) {
        self.trace.push(step);
    }

    /// Adds a warning.
    pub fn warn(&mut self, code: WarningCode, level: Level, message: impl Into<String>) {
        self.warnings.push(Warning {
            code,
            message: message.into(),
            level,
        });
    }
}

/// Why a calculation could not be registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryError(pub String);

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RegistryError {}

/// All registered calculations, by ID and version.
#[derive(Default)]
pub struct Registry {
    calcs: BTreeMap<String, BTreeMap<u32, Box<dyn Calculation>>>,
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<String> = self
            .calcs
            .iter()
            .flat_map(|(id, versions)| versions.keys().map(move |v| format!("{id}@{v}")))
            .collect();
        f.debug_struct("Registry")
            .field("calculations", &ids)
            .finish()
    }
}

impl Registry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// A registry holding every built-in calculation.
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        for calc in crate::calcs::builtins() {
            if let Err(e) = registry.register(calc) {
                // A malformed built-in descriptor is a CivKit bug caught by the test suite.
                panic!("invalid built-in calculation: {e}");
            }
        }
        registry
    }

    /// Adds a calculation after checking its descriptor is well formed.
    pub fn register(&mut self, calc: Box<dyn Calculation>) -> Result<(), RegistryError> {
        check_descriptor(calc.descriptor())?;
        let d = calc.descriptor();
        let versions = self.calcs.entry(d.id.clone()).or_default();
        if versions.contains_key(&d.version) {
            return Err(RegistryError(format!(
                "{} version {} is already registered",
                d.id, d.version
            )));
        }
        versions.insert(d.version, calc);
        Ok(())
    }

    /// Every registered descriptor, all versions, sorted by ID then version.
    pub fn list(&self) -> Vec<&Descriptor> {
        self.calcs
            .values()
            .flat_map(|v| v.values().map(|c| c.descriptor()))
            .collect()
    }

    /// The descriptor of one calculation; `version: None` means the latest.
    pub fn describe(&self, id: &str, version: Option<u32>) -> Option<&Descriptor> {
        self.find(id, version).ok().map(|c| c.descriptor())
    }

    fn find(&self, id: &str, version: Option<u32>) -> Result<&dyn Calculation, CalcError> {
        let versions = self.calcs.get(id).ok_or_else(|| {
            CalcError::new(
                ErrorCode::UnknownCalculation,
                format!("no calculation '{id}'"),
            )
        })?;
        let calc = match version {
            Some(v) => versions.get(&v).ok_or_else(|| {
                let known: Vec<String> = versions.keys().map(u32::to_string).collect();
                CalcError::new(
                    ErrorCode::UnknownVersion,
                    format!("'{id}' has no version {v}; available: {}", known.join(", ")),
                )
            })?,
            None => versions
                .values()
                .next_back()
                .ok_or_else(|| CalcError::internal(format!("'{id}' has no versions")))?,
        };
        Ok(calc.as_ref())
    }

    /// Runs a request. On failure, returns every input error found (not only the first).
    pub fn call(&self, request: &Request) -> Result<Response, ErrorEnvelope> {
        let calc = self
            .find(&request.id, request.version)
            .map_err(|e| ErrorEnvelope(vec![e]))?;
        let d = calc.descriptor();
        let internal = |e: CalcError| ErrorEnvelope(vec![e]);

        let validated = validate_inputs(d, request).map_err(ErrorEnvelope)?;
        let mut ctx = Context::default();
        apply_validity_rules(d, &validated.inputs, &mut ctx).map_err(ErrorEnvelope)?;

        let computed = catch_unwind(AssertUnwindSafe(|| {
            calc.compute(&validated.inputs, &mut ctx)
        }))
        .map_err(|_| internal(CalcError::internal(format!("'{}' panicked", d.id))))?
        .map_err(|e| ErrorEnvelope(vec![e]))?;

        // Trace order: choice steps, then the registry's conversion steps, then the computation (D-23 §3.6).
        let (choices, rest) =
            split_choice_steps(d, &validated.inputs, ctx.trace).map_err(internal)?;
        let trace: Vec<TraceStep> = choices
            .into_iter()
            .chain(validated.notes)
            .chain(rest)
            .collect();
        check_trace(d, &trace, &validated.group_lengths).map_err(internal)?;

        let mut outputs = BTreeMap::new();
        for spec in &d.outputs {
            let produced = computed.0.get(&spec.name).ok_or_else(|| {
                internal(CalcError::internal(format!(
                    "'{}' did not produce output '{}'",
                    d.id, spec.name
                )))
            })?;
            let values: Vec<f64> = match (spec.kind, produced) {
                (OutputKind::Scalar, Number::Scalar(v)) => vec![*v],
                (OutputKind::List, Number::List(v)) => {
                    let expected = spec
                        .length
                        .as_ref()
                        .and_then(|l| index_set_len(&validated.group_lengths, &l.group, l.offset));
                    if expected != Some(v.len()) || v.len() > MAX_LIST_LEN {
                        return Err(internal(CalcError::internal(format!(
                            "'{}' produced {} values for list output '{}', expected {expected:?}",
                            d.id,
                            v.len(),
                            spec.name
                        ))));
                    }
                    v.clone()
                }
                _ => {
                    return Err(internal(CalcError::internal(format!(
                        "'{}' produced the wrong kind of value (scalar or list) for output '{}'",
                        d.id, spec.name
                    ))));
                }
            };
            if let Some(k) = values.iter().position(|v| !v.is_finite()) {
                return Err(internal(CalcError::internal(format!(
                    "'{}' produced a non-finite value for '{}' (position {k})",
                    d.id, spec.name
                ))));
            }
            // Presentation unit: requested in the request, else the descriptor's default, else the SI unit.
            let presentation = request
                .output_units
                .get(&spec.name)
                .or(spec.display_unit.as_ref())
                .or(spec.si_unit.as_ref());
            let (values, unit) = match presentation {
                None => (values, None),
                // Units were validated (request) or checked at registration (descriptor); parsing cannot fail.
                Some(text) => match Unit::parse(text) {
                    Ok(unit) => (
                        values.into_iter().map(|v| unit.from_si(v)).collect(),
                        Some(text.clone()),
                    ),
                    Err(e) => return Err(internal(CalcError::internal(e.to_string()))),
                },
            };
            let value = match spec.kind {
                OutputKind::Scalar => Number::Scalar(values[0]),
                OutputKind::List => Number::List(values),
            };
            outputs.insert(spec.name.clone(), OutputValue { value, unit });
        }

        Ok(Response {
            id: d.id.clone(),
            version: d.version,
            civkit_version: crate::VERSION.to_owned(),
            outputs,
            trace,
            warnings: ctx.warnings,
            references: d.references.clone(),
            assumptions: d.assumptions.clone(),
        })
    }

    /// Runs a JSON request and always returns JSON: the response, or the error envelope.
    pub fn call_json(&self, request_json: &str) -> String {
        let result = match serde_json::from_str::<Request>(request_json) {
            Ok(request) => self.call(&request),
            Err(e) => Err(ErrorEnvelope(vec![CalcError::new(
                ErrorCode::InvalidJson,
                e.to_string(),
            )])),
        };
        let json = match result {
            Ok(response) => serde_json::to_string(&response),
            Err(envelope) => serde_json::to_string(&envelope),
        };
        json.unwrap_or_else(|e| {
            format!(r#"{{"error":{{"code":"INTERNAL","message":"could not serialise the result: {e}"}}}}"#)
        })
    }
}

/// The result of request validation.
struct Validated {
    inputs: Inputs,
    /// Visible conversions (bearings to azimuths), placed after the choice steps in the trace.
    notes: Vec<TraceStep>,
    /// The length of each group, from its lists.
    group_lengths: BTreeMap<String, usize>,
}

/// Length of the index set (`group`, `offset`), if the group is known and the result is non-negative.
fn index_set_len(lengths: &BTreeMap<String, usize>, group: &str, offset: i8) -> Option<usize> {
    let n = *lengths.get(group)?;
    usize::try_from(n as i64 + i64::from(offset)).ok()
}

/// Checks the request's inputs against the descriptor and returns them in SI, with the trace steps for
/// conversions that must be visible (bearings to azimuths) and the length of each group.
fn validate_inputs(d: &Descriptor, request: &Request) -> Result<Validated, Vec<CalcError>> {
    let mut errors = Vec::new();
    let mut notes = Vec::new();
    let known: BTreeSet<&str> = d.inputs.iter().map(|s| s.name.as_str()).collect();

    for name in request.inputs.keys() {
        if !known.contains(name.as_str()) {
            errors.push(CalcError::for_input(
                ErrorCode::UnknownInput,
                name,
                format!(
                    "'{}' has no input '{name}'; inputs: {}",
                    d.id,
                    known_list(&known)
                ),
            ));
        }
    }
    for (name, unit) in &request.output_units {
        match d.outputs.iter().find(|o| &o.name == name) {
            None => errors.push(CalcError::new(
                ErrorCode::UnknownOutput,
                format!("'{}' has no output '{name}'", d.id),
            )),
            Some(spec) => {
                if let Err(e) = Unit::parse(unit).and_then(|u| u.expect(spec.dimension)) {
                    errors.push(unit_error(&format!("output_units.{name}"), &e));
                }
            }
        }
    }

    let mut inputs = Inputs::default();
    // Raw array lengths of list inputs, by group, in declaration order (for the length checks below).
    let mut raw_lengths: BTreeMap<&str, Vec<(&str, usize)>> = BTreeMap::new();
    for spec in &d.inputs {
        let given = request.inputs.get(&spec.name);
        let Some(given) = given else {
            if spec.required {
                errors.push(CalcError::for_input(
                    ErrorCode::MissingInput,
                    &spec.name,
                    format!("required input '{}' is missing", spec.name),
                ));
            } else if let Some(default) = spec.default {
                inputs.scalars.insert(spec.name.clone(), default);
            }
            continue;
        };
        match spec.kind {
            InputKind::Scalar => match to_si(spec, given) {
                Ok((v, _)) if !spec.domain.contains(v) => errors.push(CalcError::for_input(
                    ErrorCode::OutOfDomain,
                    &spec.name,
                    format!(
                        "'{}' = {v}{} is outside its domain {}{}",
                        spec.name,
                        unit_suffix(spec.si_unit.as_deref()),
                        spec.domain,
                        unit_suffix(spec.si_unit.as_deref())
                    ),
                )),
                Ok((v, note)) => {
                    inputs.scalars.insert(spec.name.clone(), v);
                    notes.extend(note);
                }
                Err(e) => errors.push(e),
            },
            InputKind::Choice => match choice_value(spec, given) {
                Ok(v) => {
                    inputs.choices.insert(spec.name.clone(), v);
                }
                Err(e) => errors.push(e),
            },
            InputKind::List => {
                let group = spec.group.as_deref().unwrap_or_default();
                let item = d
                    .groups
                    .iter()
                    .find(|g| g.name == group)
                    .map_or("entry", |g| g.item.as_str());
                if let Some(a) = given.value.as_array() {
                    raw_lengths
                        .entry(group)
                        .or_default()
                        .push((&spec.name, a.len()));
                }
                match list_to_si(spec, given, item) {
                    Ok((values, list_notes)) => {
                        inputs.lists.insert(spec.name.clone(), values);
                        notes.extend(list_notes);
                    }
                    Err(e) => errors.extend(e),
                }
            }
        }
    }

    let mut group_lengths = BTreeMap::new();
    for g in &d.groups {
        let Some(lists) = raw_lengths.get(g.name.as_str()) else {
            continue;
        };
        let (first_name, n) = lists[0];
        for (name, len) in &lists[1..] {
            if *len != n {
                errors.push(CalcError::for_input(
                    ErrorCode::InvalidValue,
                    *name,
                    format!(
                        "group '{}': '{first_name}' has {n} entries ({}s), '{name}' has {len}; lists in one group must be the same length",
                        g.name, g.item
                    ),
                ));
            }
        }
        let largest_offset = d
            .outputs
            .iter()
            .filter_map(|o| o.length.as_ref())
            .filter(|l| l.group == g.name)
            .map(|l| l.offset.max(0) as usize)
            .max()
            .unwrap_or(0);
        let max_len = MAX_LIST_LEN - largest_offset;
        if n < g.min_len {
            errors.push(CalcError::for_input(
                ErrorCode::InvalidValue,
                first_name,
                format!(
                    "group '{}' needs at least {} entries ({}s); '{first_name}' has {n}",
                    g.name, g.min_len, g.item
                ),
            ));
        } else if n > max_len {
            errors.push(CalcError::for_input(
                ErrorCode::InvalidValue,
                first_name,
                format!(
                    "group '{}' allows at most {max_len} entries ({}s); '{first_name}' has {n}",
                    g.name, g.item
                ),
            ));
        }
        group_lengths.insert(g.name.clone(), n);
    }

    if errors.is_empty() {
        Ok(Validated {
            inputs,
            notes,
            group_lengths,
        })
    } else {
        Err(errors)
    }
}

/// Checks a choice input: a string from the declared list, with no unit (D-23).
fn choice_value(spec: &InputSpec, given: &InputValue) -> Result<String, CalcError> {
    let name = spec.name.as_str();
    let allowed = spec.choices.join(", ");
    let invalid = |msg: String| CalcError::for_input(ErrorCode::InvalidValue, name, msg);
    if given.unit.is_some() || given.negative.is_some() {
        return Err(invalid(format!(
            "'{name}' is a choice: give only \"value\", one of {allowed}"
        )));
    }
    match given.value.as_str() {
        Some(v) if spec.choices.iter().any(|c| c == v) => Ok(v.to_owned()),
        Some(v) => Err(invalid(format!(
            "'{name}' = \"{v}\" is not one of its choices: {allowed}"
        ))),
        None => Err(invalid(format!(
            "'{name}' is a choice: give a string, one of {allowed}"
        ))),
    }
}

/// Converts a list input to SI, element by element, with one unit for the whole list (D-23). Element errors
/// carry `index` and name the entry 1-based ("leg 3: …"); a problem with the list as a whole has no index.
fn list_to_si(
    spec: &InputSpec,
    given: &InputValue,
    item: &str,
) -> Result<(Vec<f64>, Vec<TraceStep>), Vec<CalcError>> {
    let name = spec.name.as_str();
    let invalid = |msg: String| CalcError::for_input(ErrorCode::InvalidValue, name, msg);
    let at = |mut e: CalcError, k: usize| {
        e.message = format!("{item} {}: {}", k + 1, e.message);
        e.index = Some(k);
        e
    };
    let Some(elements) = given.value.as_array() else {
        return Err(vec![invalid(format!(
            "'{name}' is a list: give a JSON array of values with one unit for the whole list"
        ))]);
    };
    if given.negative.is_some() {
        return Err(vec![invalid(format!(
            "'{name}': \"negative\" is not used with lists"
        ))]);
    }
    let unit = given.unit.as_deref();
    if unit == Some("dms") {
        return Err(vec![invalid(format!(
            "'{name}': lists of DMS values are not supported in 0.1; give the angles in deg or rad"
        ))]);
    }
    if unit == Some("bearing") && !spec.accepts_bearing {
        return Err(vec![CalcError::for_input(
            ErrorCode::UnitMismatch,
            name,
            format!("'{name}' does not accept quadrant bearings"),
        )]);
    }
    // One unit for the whole list: resolve it once.
    let factor = match unit {
        Some("bearing") => None,
        None if spec.dimension.is_dimensionless() => None,
        None => {
            return Err(vec![CalcError::for_input(
                ErrorCode::MissingUnit,
                name,
                format!(
                    "'{name}' has dimension {}, so the list needs a unit; CivKit never assumes one",
                    spec.dimension
                ),
            )]);
        }
        Some(text) => match Unit::parse(text).and_then(|u| u.expect(spec.dimension)) {
            Ok(u) => Some(u),
            Err(e) => return Err(vec![unit_error(name, &e)]),
        },
    };

    let mut values = Vec::with_capacity(elements.len());
    let mut notes = Vec::new();
    let mut errors = Vec::new();
    for (k, element) in elements.iter().enumerate() {
        let converted = if unit == Some("bearing") {
            match element.as_str() {
                None => Err(invalid(format!("'{name}[{k}]' must be a bearing string"))),
                Some(text) => match bearing_to_azimuth(text) {
                    Ok(azimuth) => {
                        notes.push(TraceStep {
                            step: format!(
                                "{item} {}: {name}: bearing {text} → azimuth (clockwise from north)",
                                k + 1
                            ),
                            value: Some(azimuth),
                            unit: Some("rad".into()),
                            item: Some(TraceItem {
                                label: item.to_owned(),
                                index: k,
                            }),
                            ..TraceStep::default()
                        });
                        Ok(azimuth)
                    }
                    Err(e) => Err(unit_error(name, &e)),
                },
            }
        } else {
            match element.as_f64().filter(|v| v.is_finite()) {
                None => Err(invalid(format!("'{name}[{k}]' must be a finite number"))),
                Some(v) => Ok(factor.as_ref().map_or(v, |u| u.to_si(v))),
            }
        };
        match converted {
            Ok(v) if !spec.domain.contains(v) => errors.push(at(
                CalcError::for_input(
                    ErrorCode::OutOfDomain,
                    name,
                    format!(
                        "{name}[{k}] = {v}{} is outside its domain {}{}",
                        unit_suffix(spec.si_unit.as_deref()),
                        spec.domain,
                        unit_suffix(spec.si_unit.as_deref())
                    ),
                ),
                k,
            )),
            Ok(v) => values.push(v),
            Err(e) => errors.push(at(e, k)),
        }
    }
    if errors.is_empty() {
        Ok((values, notes))
    } else {
        Err(errors)
    }
}

/// Maps a unit error to the contract's error codes (`docs/API_DESIGN.md` §5).
fn unit_error(input: &str, e: &UnitError) -> CalcError {
    let code = match e {
        UnitError::Ambiguous { .. } => ErrorCode::AmbiguousUnit,
        UnitError::Mismatch { .. } => ErrorCode::UnitMismatch,
        UnitError::InvalidValue(_) => ErrorCode::InvalidValue,
        UnitError::Empty | UnitError::Unknown { .. } | UnitError::Syntax(_) => {
            ErrorCode::UnknownUnit
        }
    };
    CalcError::for_input(code, input, format!("'{input}': {e}"))
}

/// Converts one input value to SI (`docs/API_DESIGN.md` §7). Returns a trace step when the conversion must be
/// visible in the trace (a bearing converted to an azimuth).
fn to_si(spec: &InputSpec, given: &InputValue) -> Result<(f64, Option<TraceStep>), CalcError> {
    let name = spec.name.as_str();
    let invalid = |msg: String| CalcError::for_input(ErrorCode::InvalidValue, name, msg);
    let unit = given.unit.as_deref();

    if given.negative.is_some() && unit != Some("dms") {
        return Err(invalid(format!(
            "'{name}': \"negative\" is only used with \"unit\": \"dms\""
        )));
    }
    match unit {
        Some("dms") => {
            if spec.dimension != Dimension::ANGLE {
                return Err(unit_error(
                    name,
                    &UnitError::Mismatch {
                        expected: spec.dimension,
                        found: Dimension::ANGLE,
                    },
                ));
            }
            let parts: Option<Vec<f64>> = given
                .value
                .as_array()
                .map(|a| a.iter().filter_map(serde_json::Value::as_f64).collect());
            let parts = parts
                .filter(|p| given.value.as_array().is_some_and(|a| a.len() == p.len()))
                .ok_or_else(|| {
                    invalid(format!(
                        "'{name}': a DMS value is an array [degrees, minutes, seconds]"
                    ))
                })?;
            let radians = dms_to_radians(&parts, given.negative.unwrap_or(false))
                .map_err(|e| unit_error(name, &e))?;
            Ok((radians, None))
        }
        Some("bearing") => {
            if !spec.accepts_bearing {
                return Err(CalcError::for_input(
                    ErrorCode::UnitMismatch,
                    name,
                    format!("'{name}' does not accept a quadrant bearing"),
                ));
            }
            let text = given.value.as_str().ok_or_else(|| {
                invalid(format!(
                    "'{name}': a bearing is a string such as \"N 3°45'00\\\" E\""
                ))
            })?;
            let azimuth = bearing_to_azimuth(text).map_err(|e| unit_error(name, &e))?;
            let note = TraceStep {
                step: format!("{name}: bearing {text} → azimuth (clockwise from north)"),
                value: Some(azimuth),
                unit: Some("rad".into()),
                reference: None,
                item: None,
                choice: None,
            };
            Ok((azimuth, Some(note)))
        }
        _ => {
            let number = given
                .value
                .as_f64()
                .filter(|v| v.is_finite())
                .ok_or_else(|| invalid(format!("'{name}' must be a finite number")))?;
            match unit {
                None if spec.dimension.is_dimensionless() => Ok((number, None)),
                None => Err(CalcError::for_input(
                    ErrorCode::MissingUnit,
                    name,
                    format!(
                        "'{name}' has dimension {}, so it needs a unit; CivKit never assumes one",
                        spec.dimension
                    ),
                )),
                Some(text) => {
                    let unit = Unit::parse(text)
                        .and_then(|u| u.expect(spec.dimension))
                        .map_err(|e| unit_error(name, &e))?;
                    Ok((unit.to_si(number), None))
                }
            }
        }
    }
}

/// Applies the descriptor's validity rules: warnings by default, errors where escalated.
fn apply_validity_rules(
    d: &Descriptor,
    inputs: &Inputs,
    ctx: &mut Context,
) -> Result<(), Vec<CalcError>> {
    let mut errors = Vec::new();
    for rule in &d.validity {
        let Some(value) = inputs.get(&rule.input) else {
            continue;
        };
        if rule.range.contains(value) {
            continue;
        }
        let unit = d
            .inputs
            .iter()
            .find(|i| i.name == rule.input)
            .and_then(|i| i.si_unit.as_deref());
        let message = format!(
            "'{}' = {value}{} is outside the method's validity range {}{}: {}",
            rule.input,
            unit_suffix(unit),
            rule.range,
            unit_suffix(unit),
            rule.reason
        );
        match rule.on_breach {
            Breach::Warning => ctx.warn(WarningCode::OutsideValidityRange, Level::Caution, message),
            Breach::Error => {
                errors.push(CalcError::for_input(
                    ErrorCode::OutsideValidityRange,
                    &rule.input,
                    message,
                ));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// `" m"` for a unit, `""` for dimensionless: numbers in messages always name their internal unit.
fn unit_suffix(unit: Option<&str>) -> String {
    unit.map(|u| format!(" {u}")).unwrap_or_default()
}

/// Separates the leading choice steps of `compute`'s trace from the rest and checks them (D-23 §3.6): exactly one
/// per choice input, first, in declaration order, carrying the requested value; none anywhere else.
fn split_choice_steps(
    d: &Descriptor,
    inputs: &Inputs,
    mut trace: Vec<TraceStep>,
) -> Result<(Vec<TraceStep>, Vec<TraceStep>), CalcError> {
    let id = &d.id;
    let declared: Vec<&InputSpec> = d
        .inputs
        .iter()
        .filter(|i| i.kind == InputKind::Choice)
        .collect();
    for (k, spec) in declared.iter().enumerate() {
        let wanted = inputs.choice(&spec.name)?;
        let ok = trace.get(k).is_some_and(|s| {
            s.step == spec.name
                && s.choice.as_deref() == Some(wanted)
                && s.value.is_none()
                && s.unit.is_none()
                && s.item.is_none()
        });
        if !ok {
            return Err(CalcError::internal(format!(
                "'{id}': trace step {} must record choice '{}' = \"{wanted}\" (one choice step per choice input, first, in declaration order)",
                k + 1,
                spec.name
            )));
        }
    }
    let rest = trace.split_off(declared.len());
    if let Some(extra) = rest.iter().find(|s| s.choice.is_some()) {
        return Err(CalcError::internal(format!(
            "'{id}': choice step '{}' is duplicated, undeclared or after a numeric step",
            extra.step
        )));
    }
    Ok((trace, rest))
}

/// Checks every numeric trace step: a finite value in an internal unit (coherent SI, §7.1.5), and a valid
/// `item` whose label names a declared index set, whose index is in range, and whose text prefix agrees.
/// A violation is a bug in the calculation, reported as `INTERNAL` rather than shown to users.
fn check_trace(
    d: &Descriptor,
    trace: &[TraceStep],
    group_lengths: &BTreeMap<String, usize>,
) -> Result<(), CalcError> {
    let id = &d.id;
    let index_sets = index_sets(d);
    for step in trace {
        if step.choice.is_some() {
            continue; // checked by split_choice_steps
        }
        let Some(value) = step.value.filter(|v| v.is_finite()) else {
            return Err(CalcError::internal(format!(
                "'{id}': trace step '{}' has no finite value",
                step.step
            )));
        };
        let _ = value;
        if let Some(text) = &step.unit {
            match Unit::parse(text) {
                Ok(u) if (u.factor - 1.0).abs() <= f64::EPSILON => {}
                Ok(_) => {
                    return Err(CalcError::internal(format!(
                        "'{id}': trace step '{}' uses '{text}', which is not an internal (coherent SI) unit",
                        step.step
                    )));
                }
                Err(e) => {
                    return Err(CalcError::internal(format!(
                        "'{id}': trace step '{}': {e}",
                        step.step
                    )));
                }
            }
        }
        if let Some(item) = &step.item {
            let len = index_sets
                .get(item.label.as_str())
                .and_then(|(g, off)| index_set_len(group_lengths, g, *off));
            let Some(len) = len else {
                return Err(CalcError::internal(format!(
                    "'{id}': trace step '{}' names item '{}', which is not a declared index set",
                    step.step, item.label
                )));
            };
            if item.index >= len {
                return Err(CalcError::internal(format!(
                    "'{id}': trace step '{}': {} index {} is out of range ({len} entries)",
                    step.step, item.label, item.index
                )));
            }
            let prefix = format!("{} {}: ", item.label, item.index + 1);
            if !step.step.starts_with(&prefix) {
                return Err(CalcError::internal(format!(
                    "'{id}': trace step '{}' does not start with '{prefix}'",
                    step.step
                )));
            }
        }
    }
    Ok(())
}

/// Item label → index set (group, offset): each group's own item names (group, 0); each list output's item
/// names (its group, its offset). `check_descriptor` guarantees one index set per label.
fn index_sets(d: &Descriptor) -> BTreeMap<&str, (&str, i8)> {
    let mut sets = BTreeMap::new();
    for g in &d.groups {
        sets.insert(g.item.as_str(), (g.name.as_str(), 0));
    }
    for l in d.outputs.iter().filter_map(|o| o.length.as_ref()) {
        sets.insert(l.item.as_str(), (l.group.as_str(), l.offset));
    }
    sets
}

fn known_list(known: &BTreeSet<&str>) -> String {
    known.iter().copied().collect::<Vec<_>>().join(", ")
}

/// Checks that a descriptor is well formed before it is registered.
pub fn check_descriptor(d: &Descriptor) -> Result<(), RegistryError> {
    let err = |msg: String| Err(RegistryError(format!("{}: {msg}", d.id)));
    let segments: Vec<&str> = d.id.split('.').collect();
    let segment_ok = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && s.starts_with(|c: char| c.is_ascii_lowercase())
    };
    if segments.len() != 3 || !segments.iter().all(|s| segment_ok(s)) {
        return err("ID must be 'domain.topic.method' in lowercase letters, digits and '_'".into());
    }
    if !DOMAINS.contains(&segments[0]) {
        return err(format!(
            "unknown domain '{}'; allowed: {}",
            segments[0],
            DOMAINS.join(", ")
        ));
    }
    if d.version == 0 {
        return err("versions start at 1".into());
    }
    if d.outputs.is_empty() {
        return err("a calculation needs at least one output".into());
    }
    let mut names = BTreeSet::new();
    for name in d
        .inputs
        .iter()
        .map(|i| &i.name)
        .chain(d.outputs.iter().map(|o| &o.name))
    {
        if !names.insert(name.as_str()) {
            return err(format!("name '{name}' is used twice"));
        }
    }
    for input in &d.inputs {
        if input.required && input.default.is_some() {
            return err(format!(
                "required input '{}' must not have a default",
                input.name
            ));
        }
        if let Some(default) = input.default
            && !input.domain.contains(default)
        {
            return err(format!(
                "default of '{}' is outside its own domain",
                input.name
            ));
        }
    }
    for (kind, name, dimension, si_unit, display_unit) in d
        .inputs
        .iter()
        .map(|i| ("input", &i.name, i.dimension, &i.si_unit, &i.display_unit))
        .chain(
            d.outputs
                .iter()
                .map(|o| ("output", &o.name, o.dimension, &o.si_unit, &o.display_unit)),
        )
    {
        match (si_unit, dimension.is_dimensionless()) {
            (None, true) => {}
            (None, false) => {
                return err(format!(
                    "{kind} '{name}' needs its internal SI unit (si_unit)"
                ));
            }
            (Some(text), _) => match Unit::parse(text).and_then(|u| u.expect(dimension)) {
                Ok(u) if (u.factor - 1.0).abs() <= f64::EPSILON => {}
                Ok(_) => {
                    return err(format!(
                        "si_unit of {kind} '{name}' must be coherent SI without prefix"
                    ));
                }
                Err(e) => return err(format!("si_unit of {kind} '{name}': {e}")),
            },
        }
        if let Some(text) = display_unit
            && let Err(e) = Unit::parse(text).and_then(|u| u.expect(dimension))
        {
            return err(format!("display_unit of {kind} '{name}': {e}"));
        }
    }
    for input in &d.inputs {
        if input.accepts_bearing && input.dimension != Dimension::ANGLE {
            return err(format!(
                "input '{}' accepts bearings but is not an angle",
                input.name
            ));
        }
    }
    for rule in &d.validity {
        if !d.inputs.iter().any(|i| i.name == rule.input) {
            return err(format!(
                "validity rule names unknown input '{}'",
                rule.input
            ));
        }
        if rule.on_breach == Breach::Error && rule.reason.trim().is_empty() {
            return err(format!(
                "escalated validity rule on '{}' needs a reason",
                rule.input
            ));
        }
    }
    check_lists_and_choices(d).map_err(|msg| RegistryError(format!("{}: {msg}", d.id)))?;
    if d.references.is_empty() {
        return err("at least one reference is required".into());
    }
    Ok(())
}

fn is_snake_case(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_lowercase())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The descriptor rules for groups, list inputs and outputs, and choice inputs (D-23, ADR 0011).
fn check_lists_and_choices(d: &Descriptor) -> Result<(), String> {
    let mut groups = BTreeMap::new();
    let mut sets: BTreeMap<&str, (&str, i8)> = BTreeMap::new();
    for g in &d.groups {
        if !is_snake_case(&g.name) {
            return Err(format!("group name '{}' must be snake_case", g.name));
        }
        if groups.insert(g.name.as_str(), g).is_some() {
            return Err(format!("group '{}' is declared twice", g.name));
        }
        if g.item.trim().is_empty() || g.description.trim().is_empty() {
            return Err(format!(
                "group '{}' needs an item label and a description",
                g.name
            ));
        }
        if g.min_len < 1 {
            return Err(format!("group '{}': min_len must be at least 1", g.name));
        }
        if let Some((other, _)) = sets.insert(g.item.as_str(), (g.name.as_str(), 0)) {
            return Err(format!(
                "item label '{}' is used by groups '{other}' and '{}'",
                g.item, g.name
            ));
        }
    }
    for input in &d.inputs {
        let n = &input.name;
        match input.kind {
            InputKind::Scalar => {
                if input.group.is_some() || !input.choices.is_empty() {
                    return Err(format!(
                        "scalar input '{n}' must not have a group or choices"
                    ));
                }
            }
            InputKind::List => {
                let Some(group) = &input.group else {
                    return Err(format!("list input '{n}' needs a group"));
                };
                if !groups.contains_key(group.as_str()) {
                    return Err(format!("list input '{n}' names unknown group '{group}'"));
                }
                if !input.required || input.default.is_some() || !input.choices.is_empty() {
                    return Err(format!(
                        "list input '{n}' must be required, with no default and no choices (0.1)"
                    ));
                }
            }
            InputKind::Choice => {
                if input.choices.is_empty() {
                    return Err(format!("choice input '{n}' needs at least one choice"));
                }
                let mut seen = BTreeSet::new();
                for c in &input.choices {
                    if !is_snake_case(c) || !seen.insert(c.as_str()) {
                        return Err(format!(
                            "choice input '{n}': choices must be unique lower-case snake_case ('{c}')"
                        ));
                    }
                }
                if !input.required
                    || input.default.is_some()
                    || input.group.is_some()
                    || input.accepts_bearing
                    || !input.dimension.is_dimensionless()
                    || input.si_unit.is_some()
                    || input.display_unit.is_some()
                    || !input.domain.is_unbounded()
                {
                    return Err(format!(
                        "choice input '{n}' must be required, with no default, group, unit or domain (0.1)"
                    ));
                }
            }
        }
    }
    for g in &d.groups {
        if !d
            .inputs
            .iter()
            .any(|i| i.group.as_deref() == Some(g.name.as_str()))
        {
            return Err(format!("group '{}' has no list input", g.name));
        }
    }
    for output in &d.outputs {
        let n = &output.name;
        match (output.kind, &output.length) {
            (OutputKind::Scalar, None) => {}
            (OutputKind::Scalar, Some(_)) => {
                return Err(format!("scalar output '{n}' must not have a length rule"));
            }
            (OutputKind::List, None) => {
                return Err(format!("list output '{n}' needs a length rule"));
            }
            (OutputKind::List, Some(l)) => {
                let Some(g) = groups.get(l.group.as_str()) else {
                    return Err(format!(
                        "list output '{n}' names unknown group '{}'",
                        l.group
                    ));
                };
                if !(-1..=1).contains(&l.offset) {
                    return Err(format!("list output '{n}': offset must be -1, 0 or +1"));
                }
                if (g.min_len as i64) + i64::from(l.offset) < 1 {
                    return Err(format!(
                        "list output '{n}' could be empty: group '{}' allows {} entries and the offset is {}",
                        g.name, g.min_len, l.offset
                    ));
                }
                if l.item.trim().is_empty() || output.description.trim().is_empty() {
                    return Err(format!(
                        "list output '{n}' needs an item label and a description"
                    ));
                }
                let set = (l.group.as_str(), l.offset);
                match sets.get(l.item.as_str()) {
                    Some(existing) if *existing != set => {
                        return Err(format!(
                            "item label '{}' of list output '{n}' already names another index set ({}, {:+})",
                            l.item, existing.0, existing.1
                        ));
                    }
                    Some(_) => {}
                    None => {
                        sets.insert(l.item.as_str(), set);
                    }
                }
            }
        }
    }
    for rule in &d.validity {
        if d.inputs
            .iter()
            .any(|i| i.name == rule.input && i.kind != InputKind::Scalar)
        {
            return Err(format!(
                "validity rules apply to scalar inputs only ('{}')",
                rule.input
            ));
        }
    }
    Ok(())
}

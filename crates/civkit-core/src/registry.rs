// Provenance: Claude Opus 5.5 (claude-opus-5-5) · first drafted 2026-11-25 10:58 IST · last edited 2026-11-25 15:57 IST

//! The calculation trait and the registry that validates requests and runs calculations.

use crate::contract::{
    InputValue, Level, OutputValue, Request, Response, TraceStep, Warning, WarningCode,
};
use crate::descriptor::{Breach, Descriptor, InputSpec};
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

/// Validated input values, in SI, by name.
#[derive(Debug, Clone, Default)]
pub struct Inputs(BTreeMap<String, f64>);

impl Inputs {
    /// The value of an input, if present (optional inputs without a default may be absent).
    pub fn get(&self, name: &str) -> Option<f64> {
        self.0.get(name).copied()
    }

    /// The value of a required input. Returns `INTERNAL` if absent, which would be a CivKit bug,
    /// because the registry checks required inputs before `compute` runs.
    pub fn require(&self, name: &str) -> Result<f64, CalcError> {
        self.get(name).ok_or_else(|| {
            CalcError::internal(format!("required input '{name}' reached compute() missing"))
        })
    }
}

/// Output values, in SI, by name.
#[derive(Debug, Clone, Default)]
pub struct Outputs(BTreeMap<String, f64>);

impl Outputs {
    /// An empty set of outputs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an output (builder style).
    #[must_use]
    pub fn with(mut self, name: &str, value: f64) -> Self {
        self.0.insert(name.to_owned(), value);
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
    /// Records one step of the trace. `unit` is the SI unit, or `None` when dimensionless.
    pub fn step(&mut self, expression: &str, value: f64, unit: Option<&str>) {
        self.trace.push(TraceStep {
            step: expression.to_owned(),
            value,
            unit: unit.map(str::to_owned),
            reference: None,
        });
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

        let (inputs, notes) = validate_inputs(d, request).map_err(ErrorEnvelope)?;
        let mut ctx = Context {
            trace: notes,
            warnings: Vec::new(),
        };
        apply_validity_rules(d, &inputs, &mut ctx).map_err(ErrorEnvelope)?;

        let computed = catch_unwind(AssertUnwindSafe(|| calc.compute(&inputs, &mut ctx)))
            .map_err(|_| ErrorEnvelope(vec![CalcError::internal(format!("'{}' panicked", d.id))]))?
            .map_err(|e| ErrorEnvelope(vec![e]))?;

        let mut outputs = BTreeMap::new();
        for spec in &d.outputs {
            let value = computed.0.get(&spec.name).copied().ok_or_else(|| {
                ErrorEnvelope(vec![CalcError::internal(format!(
                    "'{}' did not produce output '{}'",
                    d.id, spec.name
                ))])
            })?;
            if !value.is_finite() {
                return Err(ErrorEnvelope(vec![CalcError::internal(format!(
                    "'{}' produced a non-finite value for '{}'",
                    d.id, spec.name
                ))]));
            }
            // Presentation unit: requested in the request, else the descriptor's default, else the SI unit.
            let presentation = request
                .output_units
                .get(&spec.name)
                .or(spec.display_unit.as_ref())
                .or(spec.si_unit.as_ref());
            let output = match presentation {
                None => OutputValue { value, unit: None },
                // Units were validated (request) or checked at registration (descriptor); parsing cannot fail.
                Some(text) => match Unit::parse(text) {
                    Ok(unit) => OutputValue {
                        value: unit.from_si(value),
                        unit: Some(text.clone()),
                    },
                    Err(e) => return Err(ErrorEnvelope(vec![CalcError::internal(e.to_string())])),
                },
            };
            outputs.insert(spec.name.clone(), output);
        }

        Ok(Response {
            id: d.id.clone(),
            version: d.version,
            civkit_version: crate::VERSION.to_owned(),
            outputs,
            trace: ctx.trace,
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

/// Checks the request's inputs against the descriptor and returns them as SI values, together with trace
/// steps for conversions that must be visible (bearings to azimuths).
fn validate_inputs(
    d: &Descriptor,
    request: &Request,
) -> Result<(Inputs, Vec<TraceStep>), Vec<CalcError>> {
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

    let mut values = BTreeMap::new();
    for spec in &d.inputs {
        match request.inputs.get(&spec.name) {
            None if spec.required => errors.push(CalcError::for_input(
                ErrorCode::MissingInput,
                &spec.name,
                format!("required input '{}' is missing", spec.name),
            )),
            None => {
                if let Some(default) = spec.default {
                    values.insert(spec.name.clone(), default);
                }
            }
            Some(given) => match to_si(spec, given) {
                Ok((v, _)) if !spec.domain.contains(v) => errors.push(CalcError::for_input(
                    ErrorCode::OutOfDomain,
                    &spec.name,
                    format!(
                        "'{}' = {v} (SI) is outside its domain {}",
                        spec.name, spec.domain
                    ),
                )),
                Ok((v, note)) => {
                    values.insert(spec.name.clone(), v);
                    notes.extend(note);
                }
                Err(e) => errors.push(e),
            },
        }
    }

    if errors.is_empty() {
        Ok((Inputs(values), notes))
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
                value: azimuth,
                unit: Some("rad".into()),
                reference: None,
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
        let message = format!(
            "'{}' = {value} is outside the method's validity range {}: {}",
            rule.input, rule.range, rule.reason
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
    if d.references.is_empty() {
        return err("at least one reference is required".into());
    }
    Ok(())
}

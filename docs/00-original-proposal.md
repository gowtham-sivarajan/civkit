CivKit

An Open-Source, Offline-First Engineering Toolkit

Project Proposal · Initial Concept

Tagline: Engineering tools that work where engineers work.

---

1. The Vision

CivKit is a free, open-source engineering toolkit designed to integrate directly into the software engineers already use.

Rather than creating yet another standalone engineering application, CivKit aims to provide a unified engineering computation and utility layer that can operate across multiple platforms.

Whether an engineer is working in AutoCAD, Civil 3D, MATLAB, Jupyter Notebook or an AI-assisted development environment, CivKit should make useful engineering tools accessible without requiring separate, disconnected applications.

The standalone application will exist, but it is only one part of the ecosystem.

Our ambition is to build an extensible, community-driven engineering platform, beginning with Civil Engineering.

2. The Problem

Engineering students and professionals frequently work across several disconnected applications.

- Engineering calculations are repeated manually.
- Small utilities are scattered across websites, spreadsheets and scripts.
- CAD workflows often require repetitive manual operations.
- Engineering data moves between applications with limited interoperability.
- AI tools lack a consistent interface to reliable engineering calculations.
- Many existing solutions depend on accounts, subscriptions or internet connectivity.

CivKit aims to address this fragmentation through one reusable, extensible engineering core.

3. The Proposed Solution

CivKit will consist of four major components.

A. CivKit Core

The computational foundation of the ecosystem.

It will contain reusable engineering functions, mathematical operations, unit conversions, validation systems and calculation modules.

Initial engineering domains:

- Geotechnical Engineering
- Structural Engineering
- Surveying
- Fluid Mechanics
- Construction and Quantity Estimation
- Engineering Mathematics

Rust is the proposed primary implementation language.

B. Software Integrations

CivKit should integrate into existing engineering applications rather than force users to change their workflows.

Initial targets:

Platform| Proposed integration
AutoCAD| Native C# plugin
Civil 3D| Autodesk integration
MATLAB| Local adapter
Jupyter Notebook| Python bindings
AI coding agents| MCP server

The objective is to make CivKit's engineering capabilities accessible from within these applications.

C. MCP Integration

CivKit will expose selected engineering capabilities through the Model Context Protocol.

This allows compatible AI clients to discover and use engineering tools through a standardized interface.

Potential capabilities include:

- Engineering calculations
- Unit conversions
- Drawing information retrieval
- Quantity extraction
- Calculation explanations
- Engineering data analysis

MCP will be an optional interface, not a requirement for using CivKit.

D. Standalone Application

CivKit will also provide an independent desktop application.

It will serve users who do not use supported engineering software and provide a central interface for accessing CivKit's tools.

Proposed technology: Tauri with a Rust backend.

4. Architectural Philosophy

CivKit will follow a core-first, modular architecture.

One computational core. Multiple interfaces.

Rust will power the engineering engine, while individual adapters will connect it to different applications.

Python will provide access for scientific computing and notebooks. C# will handle Autodesk integrations. MCP will expose capabilities to compatible AI clients.

This avoids maintaining separate implementations of the same engineering calculations.

5. Non-Negotiable Principles

CivKit will follow a strict independence-first philosophy.

Completely free and open source

- No paid subscriptions.
- No mandatory paid APIs.
- No proprietary cloud dependencies.

Local-first

- Core functionality works offline.
- No mandatory accounts or authentication.
- No remote database or hosted backend.
- No mandatory internet connectivity.

User-controlled

- Data remains on the user's machine.
- Local storage and export options.
- No mandatory telemetry.
- No subscription or online licence verification.

Extensible

- Documented APIs.
- Modular integrations.
- External contributions encouraged.
- New engineering domains can be added independently.

The intended result is software that users can install, inspect, modify and operate independently.

6. Proposed Technology Stack

Component| Technology
Engineering core| Rust
Python integration| PyO3
MCP| Rust MCP SDK
AutoCAD / Civil 3D| C# / .NET
Desktop application| Tauri
Local database| SQLite
Version control| Git
Public repository| GitHub
Testing| Automated unit and integration tests

All proposed development tools and dependencies must be available without paid subscriptions.

The project will not require cloud hosting to function.

7. Initial Development Scope

The first release will focus on establishing a reliable foundation rather than implementing every possible engineering utility.

CivKit 0.1 — Initial MVP

Engineering Core

- Unit conversion and dimensional validation.
- Geometry calculations.
- Basic engineering calculation modules.
- Consistent input and output structures.

AutoCAD Integration

- Read selected drawing entities.
- Extract lengths, dimensions and areas.
- Identify drawing-unit inconsistencies.
- Generate quantity schedules.
- Export extracted information.

MCP Integration

- Expose core calculations as tools.
- Provide access to supported drawing information.
- Enable compatible AI clients to use CivKit locally.

Python Integration

- Import CivKit functions into Jupyter.
- Reuse engineering calculations.
- Generate reproducible computational workflows.

The first release will prioritize read-only drawing analysis. Drawing modifications will require explicit user confirmation.

8. Development Roadmap

Phase| Objective
1| Architecture and Rust core
2| Engineering units, calculations and validation
3| Local MCP server
4| AutoCAD / Civil 3D integration
5| Python and Jupyter integration
6| Standalone desktop application
7| Expanded engineering modules and community contributions

Development will be incremental, with every module independently tested and documented.

9. Open-Source Community

CivKit is intended to be a community-driven project.

Potential contributors include:

- Engineering students.
- Civil Engineering professionals.
- Software developers.
- Computational scientists.
- Researchers and educators.

Contributors should be able to develop individual engineering modules or application integrations without needing to understand the entire codebase.

The project will use public documentation, contribution guidelines, issue tracking and transparent development.

An OSI-approved licence will be selected before the first public release.

10. Long-Term Vision

Civil Engineering is the starting point, not necessarily the limit.

The architecture should eventually support additional engineering disciplines, including Mechanical, Electrical and other computational engineering domains.

The broader vision is to create a shared, free engineering software ecosystem that improves interoperability between engineering applications.

Rather than replacing established engineering software, CivKit aims to make those tools more capable, connected and accessible.

---

CivKit — Engineering without boundaries.
Free. Open source. Local-first. Community-driven.

Status: Proposed project. Architecture, scope and implementation details subject to validation during development.
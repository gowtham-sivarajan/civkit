# Accessibility

> **Provenance:** Claude Opus 5.5 (`claude-opus-5-5`) · first drafted 2026-11-25 10:35 IST · last edited 2026-11-25 10:35 IST

CivKit should be usable by every engineer and student, including people who use screen readers, keyboard-only navigation,
magnification or high-contrast settings.

## Where the project stands

CivKit is in **Phase 0 (planning)**: there is no released software yet. This statement says what we commit to as each interface is built.
It will be updated when each interface ships ([ROADMAP.md](ROADMAP.md)).

## Commitments

- **Every result is available as text.** Each calculation returns its outputs, steps, warnings and references as
  structured text (JSON), never only as a picture or colour. Python, Jupyter and MCP all expose this text.
  Drawing analysis reports layers, lengths, areas and unit problems as text tables, so the content of a drawing
  can be reviewed without seeing it.
- **No meaning carried by colour alone.** Warnings and errors always have a code and a message, not only a colour.
- **Documentation:**
  - plain-language Markdown with real headings
  - tables with header rows
  - descriptive link text
  - alt text for any image or diagram
- **Desktop app (planned, 0.3):**
  - Target: WCAG 2.2 level AA for its interface.
  - Full keyboard operation and visible focus.
  - Labelled controls for screen readers.
  - Respects the operating system's text size, high-contrast and reduced-motion settings.
  - Tested with NVDA (Windows) and VoiceOver (macOS) before release.
- **Command-line and MCP output:** plain text or JSON, with no colour-only or animation-only information.

## Known limitations

- CAD drawings are visual by nature. CivKit can describe what a drawing contains, but it cannot make the drawing itself accessible.
- The AutoCAD and Civil 3D plugins (0.2) run inside Autodesk products. Their accessibility is limited by those hosts.
- Interfaces and documentation are in English only for now.
- No interface has been tested with assistive technology yet, because none has been built.

## Reporting a barrier

If something in CivKit or its documentation is hard or impossible for you to use:

- Open an issue using the **Bug report** form and start the title with `[Accessibility]`, or
- email the maintainer at **gowthamsivarajan@gmail.com** if you prefer not to post publicly.

Please describe what you were trying to do, what got in the way, and the assistive technology you use (if any).
Accessibility barriers are treated as bugs, not feature requests.

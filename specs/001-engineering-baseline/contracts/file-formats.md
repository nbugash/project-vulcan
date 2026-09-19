# Contract: File Formats

**Branch**: `001-engineering-baseline` | **Date**: 2026-09-18

Artifacts the gates read and write. Each is consumed by something other than its producer,
which is what makes it a contract rather than an implementation detail.

## Extracted design values

**Produced by**: `gate-fidelity extract`
**Consumed by**: `gate-fidelity lint`, the product's generated token module
**Path**: `tools/gate-fidelity/out/tokens.json`

```json
{
  "prototype_digest": "sha256:...",
  "extracted_at": "2026-09-18T00:00:00Z",
  "values": [
    {
      "name": "color-accent",
      "value": "#9184d9",
      "kind": "Colour",
      "source": "Stylesheet",
      "source_location": "mockups/_ds/nocturne-.../styles.css:42"
    }
  ]
}
```

Ordering is stable: sorted by `name`. Two runs over an unchanged prototype must produce
byte-identical files, so that a diff means the prototype moved.

## Budget measurement report

**Produced by**: `gate-budget`
**Consumed by**: the build, and the next run as its baseline
**Path**: `tools/gate-budget/out/measurements.json`

```json
{
  "runner": "AppleSilicon",
  "authoritative": true,
  "core_topology": "2P+4E",
  "captured_at": "2026-09-18T00:00:00Z",
  "measurements": [
    {
      "metric": "KeystrokeToPaint",
      "measured": { "value": 6.2, "unit": "ms" },
      "budget": { "value": 8.0, "unit": "ms" },
      "round_trip_ms": 0,
      "verdict": "Pass"
    }
  ]
}
```

`core_topology` is recorded because a measurement is only comparable to another taken on the
same topology. A baseline from a different topology is rejected rather than compared.

## Fidelity reference

**Produced by**: `gate-fidelity capture-reference`
**Consumed by**: `gate-fidelity compare`
**Path**: `tools/gate-fidelity/reference/<composition>.png` and `.json` beside it

```json
{
  "viewport": { "width": 1440, "height": 900 },
  "typefaces": ["Inter 400/500/600/700", "Phosphor 2.1.1"],
  "prototype_digest": "sha256:...",
  "environment_digest": "sha256:...",
  "captured_at": "2026-09-18T00:00:00Z"
}
```

Both digests are recorded so that a comparison failure can name its cause: a changed
prototype, a changed environment, or a changed product.

## Verification screenshot

**Produced by**: `shell-preview --screenshot`
**Consumed by**: a person, at review time
**Path**: `reports/screenshots/<feature-identity>-<title-stub>.png`

```text
reports/screenshots/F000-engineering-baseline.png            # default composition
reports/screenshots/F000-engineering-baseline-palette.png    # a notable state
reports/screenshots/F000-engineering-baseline-compact.png
reports/screenshots/F000-engineering-baseline-roomy.png
```

The identity is the feature's `F` number rather than its specification directory number,
because the `F` number is immutable while directory numbers are assigned in the order features
are specified and can diverge from it.

This is not the fidelity reference. That one lives under `tools/gate-fidelity/reference/`, is
captured inside the pinned environment, and is compared byte for byte by a machine. This one is
captured on a real machine so a person can look at the product without building the branch. A
feature that changes the interface and leaves no screenshot has not finished.

## Discrepancy document

**Produced by**: a person
**Consumed by**: review, and feature design documents by reference
**Path**: `reports/mockups-discrepancies/<id>.md`

```markdown
# <id>: <short title>

**Status**: Untriaged | Accepted | BacklogEntry
**Backlog entry**: F0NN, when status is BacklogEntry

## What we are trying to match
<the prototype behaviour or appearance>

## Why we cannot match it
<the constraint that prevents it>

## Alternatives
1. <option> — <consequence>
2. <option> — <consequence>
```

The three headings are fixed because review reads them as a set: a discrepancy without
alternatives is a complaint, and one without a reason is an excuse.

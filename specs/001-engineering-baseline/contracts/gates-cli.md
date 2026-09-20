# Contract: Gate Command-Line Interfaces

**Branch**: `001-engineering-baseline` | **Date**: 2026-09-18

Each gate is a command. These are the contracts the build and an engineer both invoke, and
FR-019 requires that the same invocation produce the same verdict in both places.

## Shared conventions

| Aspect | Contract |
| --- | --- |
| Exit `0` | The gate passed, or does not apply and said so. |
| Exit `2` | The gate failed. The build must stop. |
| Exit `1` | Usage or environment error. The gate could not run and did not judge. |
| `--json` | Emit a machine-readable report on stdout. Human text otherwise. |
| `--report <path>` | Additionally write the report to a file. |
| stderr | Diagnostics only. Never the verdict. |

Exit `1` is deliberately distinct from exit `2`: a gate that could not run has not passed,
and must never be mistaken for one that did.

## `gate-boundary`

```text
gate-boundary [--manifest-path <path>] [--json] [--report <path>]
```

| Aspect | Contract |
| --- | --- |
| Reads | Cargo workspace manifests from `--manifest-path`, default the repository root. |
| Judges | Every `LayerEdge` in the workspace graph. |
| Fails when | An edge points outward, the graph contains a cycle, or a crate declares no layer. |
| Report contains | Each offending edge: `from`, `to`, declared layers, and the manifest file and line. |
| Exit `1` when | A manifest cannot be parsed, or the workspace cannot be resolved. |

A crate with no declared layer is a failure rather than a default, so that adding a crate is a
deliberate act rather than an accidental hole in the rule.

## `gate-budget`

```text
gate-budget --runner <apple-silicon|linux-cgroup>
            [--rtt <0,10,30,80>] [--baseline <path>] [--json] [--report <path>]
```

| Aspect | Contract |
| --- | --- |
| Reads | The product binary, the constitutional budgets, and the previous accepted measurement set from `--baseline`. |
| Produces | One `BudgetMeasurement` per metric per requested round-trip profile. |
| Fails when | Any metric exceeds its budget on the authoritative runner. |
| Reports without failing when | A metric is within budget but more than ten percent worse than baseline. |
| Exit `1` when | The runner cannot enforce its constraints; the gate refuses to measure rather than report unconstrained numbers. |
| Report contains | Observed core topology, runner identity, whether the run was authoritative, and every measurement with its verdict. |

Absent `--rtt`, every profile is run. A metric that is not round-trip sensitive is measured
once and reported with `round_trip_ms` of zero.

## `gate-fidelity`

```text
gate-fidelity <extract|lint|compare|capture-reference>
              [--prototype <path>] [--manifest <path>]
              [--reference <path>] [--json] [--report <path>]
```

| Subcommand | Contract |
| --- | --- |
| `extract` | Reads the stylesheet and the asset manifest, emits the `DesignValue` set. Deterministic: identical inputs produce byte-identical output. |
| `lint` | Fails on any design literal in the source that is not in the extracted set, reporting value, file and line. |
| `compare` | Renders the target and compares against the `FidelityReference`. Any pixel difference fails. Produces a visual report on failure, not only a count. |
| `capture-reference` | Writes a new `FidelityReference`. Refuses while any declared typeface is absent from the repository. |

| Aspect | Contract |
| --- | --- |
| Exit `1` when | Run outside the pinned comparison environment for `compare` or `capture-reference`. |
| Stale reference | `compare` reports that the prototype digest no longer matches, distinguishing a moved prototype from a regressed product. |

`compare` and `capture-reference` refuse to run outside the pinned environment rather than
producing an advisory result, because an exact comparison taken elsewhere is not weaker
evidence, it is different evidence.

## `featuremap` (pre-existing, gate 9)

```text
feature_map.py <resolve|verify|add|record-spec> [...] [--json] [--map <path>]
```

Documented in `.specify/extensions/featuremap/README.md`. Its contract is unchanged by this
feature and is restated here only so the gate set is complete in one place. Exit codes already
match the shared conventions above, which is why those conventions were written to match it.

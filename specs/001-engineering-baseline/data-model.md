# Data Model: Engineering Baseline

**Branch**: `001-engineering-baseline` | **Date**: 2026-09-18 | **Plan**: [plan.md](./plan.md)

Entities this feature introduces. Field-level definitions live here and nowhere else;
`design.md` links to them rather than restating them.

## DesignValue

One named appearance decision taken from the signed-off prototype. The only permitted source
of appearance in the product.

| Field | Type | Rules |
| --- | --- | --- |
| `name` | string | Token name as written in the source, for example `color-accent`. Unique across the set. |
| `value` | string | Literal as written, for example `#9184d9` or `8px`. |
| `kind` | enum | `Colour`, `Typeface`, `Space`, `Radius`, `Shadow`, `Density`. |
| `source` | enum | `Stylesheet` or `Manifest`. Records which of the two extraction sources it came from. |
| `source_location` | string | File path and line, so a reviewer can find it. |

**Validation**: `name` matches `[a-z][a-z0-9-]*`. A duplicate `name` across sources is an
extraction failure, not a merge: the two sources disagreeing means the appearance contract is
ambiguous and must be fixed at the source.

**Relationships**: referenced by `PrototypeExtension`. Consumed by the off-token lint as the
allowed set.

## BudgetMeasurement

One metric measured in one run on the constrained machine.

| Field | Type | Rules |
| --- | --- | --- |
| `metric` | enum | One of the thirteen budgets the constitution defines. |
| `measured` | quantity | Value with unit; milliseconds, megabytes or percent. |
| `budget` | quantity | The constitutional budget for that metric, same unit. |
| `verdict` | enum | `Pass`, `Fail`, `Regressed`. `Regressed` means under budget but more than ten percent worse than the previous accepted measurement. |
| `round_trip_ms` | integer | Simulated delay in force: 0, 10, 30 or 80. |
| `runner` | enum | `AppleSilicon` or `LinuxCgroup`. |
| `authoritative` | boolean | True for `AppleSilicon`, false for `LinuxCgroup`. |
| `core_topology` | string | Performance and efficiency core counts as observed at run time. |
| `captured_at` | timestamp | UTC, ISO 8601. |

**Validation**: a measurement whose runner could not enforce its constraints is not recorded
with a failing verdict; it is not recorded at all, and the run fails. Reporting an
unconstrained number is what FR-005 forbids.

**State transitions**: `Pass` and `Regressed` allow a merge; `Regressed` requires an
explanation in the pull request. `Fail` blocks unless an exception is recorded.

**Relationships**: a run produces one measurement per metric per round-trip profile per
runner.

## FidelityReference

A stored rendering of the prototype against which later work is compared.

| Field | Type | Rules |
| --- | --- | --- |
| `image` | file | Lossless raster of the prototype composition. |
| `viewport` | dimensions | Width and height in pixels, matching the prototype's composition. |
| `typefaces` | list of string | Family and version of every typeface present at capture. |
| `prototype_digest` | string | Content hash of the prototype file the reference was taken from. |
| `environment_digest` | string | Identifier of the pinned comparison image. |
| `captured_at` | timestamp | UTC, ISO 8601. |

**Validation**: capture is refused while any typeface the prototype declares is absent from
the repository. A reference whose `prototype_digest` no longer matches the prototype is stale,
and comparison reports that the prototype moved rather than that the product regressed.

**Relationships**: one reference per composition. Compared against by the fidelity gate.

## Discrepancy

A recorded departure from the prototype, living as one document under
`reports/mockups-discrepancies/`.

| Field | Type | Rules |
| --- | --- | --- |
| `id` | string | Stable identifier used to reference it from a design document. |
| `target` | string | What was being matched. |
| `reason` | string | Why it cannot be matched. |
| `alternatives` | list of string | Options available, one of which was taken. |
| `triage` | enum | `BacklogEntry`, `Accepted`, or `Untriaged`. |
| `backlog_entry` | string or null | Feature identity when `triage` is `BacklogEntry`. |

**Validation**: `Untriaged` is a transient state; a discrepancy that remains untriaged at
review time is a blocking finding. Loosening the comparison is never a resolution.

**Relationships**: referenced by feature design documents by path. Never restated in them.

## InterfaceRegion

One region the prototype composes, rendered by the shell.

| Field | Type | Rules |
| --- | --- | --- |
| `name` | enum | `Toolbar`, `RemoteBanner`, `Rail`, `ToolWindow`, `TabStrip`, `Breadcrumbs`, `EditorSurface`, `Dock`, `DockTabStrip`, `StatusBar`, `Palette`, `Dialog`. |
| `fixed_extent` | quantity or null | The dimension the prototype's manifest fixes, where it fixes one. |
| `density_dependent` | boolean | True when its dimension changes with the density property. |
| `interactive` | boolean | True when the region responds to input in this feature. |

**Validation**: every region the prototype composes must be present; an absent region fails
SC-009. A region whose rendered dimension differs from `fixed_extent` fails the fidelity
comparison before it fails anything else.

**Relationships**: rendered by the shell; measured by the fidelity gate.

## DensityProfile

The three named density settings and the dimensions each resolves.

| Field | Type | Rules |
| --- | --- | --- |
| `name` | enum | `Compact`, `Default`, `Roomy`. |
| `code_line_height` | quantity | From the prototype manifest. |
| `row_height` | quantity | From the prototype manifest. |
| `ui_font_size` | quantity | From the prototype manifest. |
| `code_font_size` | quantity | From the prototype manifest. |
| `tool_window_width` | quantity | From the prototype manifest. |
| `dock_height` | quantity | From the prototype manifest, capped against viewport height. |

**Validation**: values are read from the manifest rather than written in code, so a manifest
change surfaces as a token diff. A profile missing any dimension is an extraction failure.

**State transitions**: switching profile re-resolves every dependent dimension within the frame
budget; no reload occurs.

## PrototypeExtension

An interface state designed because the prototype does not depict it.

| Field | Type | Rules |
| --- | --- | --- |
| `description` | string | The state, for example empty, error, or a window width the prototype does not compose. |
| `design_values` | list of `DesignValue.name` | Every value the extension uses. |
| `sign_off` | enum | `Pending`, `Approved`, `Rejected`. |
| `requested_at` | timestamp | UTC, ISO 8601. |

**Validation**: every referenced name must exist in the extracted set. An extension
introducing a value outside that set is rejected, which is the same rule the off-token lint
applies to code.

**Relationships**: references `DesignValue`. Recorded in the owning feature's design document.

## LayerEdge

One declared dependency between workspace crates, as read from the manifests.

| Field | Type | Rules |
| --- | --- | --- |
| `from` | string | Crate declaring the dependency. |
| `to` | string | Crate depended upon. |
| `declared_layer_from` | enum | `Domain`, `Application`, `Adapters`, `Composition`, `Tooling`. |
| `declared_layer_to` | enum | Same set. |
| `permitted` | boolean | Derived: true when the edge points inward or stays within a layer. |

**Validation**: the graph must be acyclic. An edge whose `permitted` is false fails gate 1
and is reported with both crate names and the manifest line that declares it.

**Relationships**: the set of edges is the workspace dependency graph.

## BacklogEntry

A feature in the ordered backlog. Owned by the pre-existing `featuremap` tooling; described
here because this feature's specification covers gate 9.

| Field | Type | Rules |
| --- | --- | --- |
| `identity` | string | `F` followed by three digits. Immutable, never reused. |
| `position` | integer | Index in file order. Conveys build order. |
| `slug` | string | Lowercase kebab-case name. |
| `depends_on` | list of identity | Each must be positioned above this entry. |
| `parallel` | boolean | Declared independent of neighbours. |
| `done` | boolean | Evidence that the work is complete, not an intention. |
| `spec_path` | string or null | The specification directory this entry produced. |
| `subfeatures` | list of (done, text) | Mergeable slices. |

**Validation**: an entry marked `done` with any unfinished subfeature is an integrity failure,
as is a duplicate identity, an unknown dependency, or a dependency positioned below its
dependent.

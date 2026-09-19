"""Behaviour tests for the feature-map extension's script."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

# Vendored beside the script it tests. The extension is maintained in the
# spec-kit repository, but the copy this repository runs is the one that decides
# whether a feature may be specified, so it is the copy that must be tested.
SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "python" / "feature_map.py"


def _load_module():
    spec = importlib.util.spec_from_file_location("feature_map", SCRIPT)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    # Register before executing: the module's dataclasses resolve their own
    # module from sys.modules while the class body is being processed.
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


feature_map = _load_module()
parse = feature_map.parse
resolve = feature_map.resolve
verify = feature_map.verify
FeatureMapError = feature_map.FeatureMapError

MAP = """# Feature Map

## Tier 0

- [x] **F000 engineering-baseline**
  - Spec: specs/001-engineering-baseline
  - [x] Import-boundary lint rule
  - [x] Budget harness

## Tier 1

- [ ] **F001 plugin-host-runtime** (depends: F000)
  - Spec: not yet specified
  - [ ] Plugin manifest format
  - [ ] Extension point registry
- [ ] **F002 app-shell-workspace** (depends: F001)
  - Spec: not yet specified
  - [ ] Main window
- [ ] **F003 remote-workspace-session** [P] (depends: F000)
  - Spec: not yet specified
  - [ ] SSH connection lifecycle
"""


@pytest.fixture()
def map_file(tmp_path: Path) -> Path:
    path = tmp_path / "feature-map.md"
    path.write_text(MAP, encoding="utf-8")
    return path


def _add(path: Path, **overrides):
    kwargs = {
        "slug": "workspace-indexing",
        "after": "F001",
        "depends": ["F000"],
        "parallel": False,
        "subfeatures": ["Incremental index build"],
        "dry_run": False,
    }
    kwargs.update(overrides)
    features = parse(path.read_text(encoding="utf-8"))
    return feature_map.add_feature(path, features, **kwargs)


class TestParse:
    def test_reads_identity_state_dependencies_and_spec_path(self):
        features = parse(MAP)
        assert [f.identity for f in features] == ["F000", "F001", "F002", "F003"]
        assert features[0].done
        assert features[0].spec_path == "specs/001-engineering-baseline"
        assert features[1].spec_path is None
        assert features[1].depends == ["F000"]
        assert features[3].parallel
        assert len(features[1].subfeatures) == 2

    def test_a_file_with_no_feature_rows_is_an_error(self):
        with pytest.raises(FeatureMapError):
            parse("# Feature Map\n\nNothing here yet.\n")


class TestResolve:
    def test_no_target_picks_first_unchecked_with_satisfied_dependencies(self):
        result = resolve(parse(MAP), None)
        assert result["STATUS"] == "ready"
        assert result["FEATURE"]["FEATURE_ID"] == "F001"

    def test_no_target_skips_features_whose_dependencies_are_incomplete(self):
        blocked = MAP.replace("- [x] **F000", "- [ ] **F000")
        assert resolve(parse(blocked), None)["FEATURE"]["FEATURE_ID"] == "F000"

    def test_file_order_decides_between_two_ready_features(self):
        advanced = MAP.replace("- [ ] **F001", "- [x] **F001").replace(
            "  - [ ] Plugin manifest format\n  - [ ] Extension point registry",
            "  - [x] Plugin manifest format\n  - [x] Extension point registry",
        )
        assert resolve(parse(advanced), None)["FEATURE"]["FEATURE_ID"] == "F002"

    def test_explicit_target_with_incomplete_dependency_is_blocked(self):
        result = resolve(parse(MAP), "F002")
        assert result["STATUS"] == "blocked"
        assert "F001" in result["REASON"]

    def test_explicit_target_already_complete_is_blocked(self):
        assert resolve(parse(MAP), "F000")["STATUS"] == "blocked"

    def test_unknown_target_is_an_error(self):
        assert resolve(parse(MAP), "F999")["STATUS"] == "error"

    def test_target_is_case_insensitive(self):
        assert resolve(parse(MAP), "f001")["STATUS"] == "ready"


class TestVerify:
    def test_clean_map_has_no_problems(self):
        assert verify(parse(MAP)) == []

    def test_checked_feature_with_unchecked_subfeature_is_reported(self):
        problems = verify(parse(MAP.replace("- [ ] **F001", "- [x] **F001")))
        assert any("unchecked subfeatures" in problem for problem in problems)

    def test_dependency_positioned_below_is_reported(self):
        broken = MAP.replace(
            "**F001 plugin-host-runtime** (depends: F000)",
            "**F001 plugin-host-runtime** (depends: F003)",
        )
        assert any("positioned" in problem for problem in verify(parse(broken)))

    def test_unknown_dependency_is_reported(self):
        broken = MAP.replace("(depends: F000)", "(depends: F404)", 1)
        assert any("unknown dependency F404" in problem for problem in verify(parse(broken)))

    def test_duplicate_identity_is_reported(self):
        broken = MAP.replace("**F002 app-shell-workspace**", "**F001 app-shell-workspace**")
        assert any("declared twice" in problem for problem in verify(parse(broken)))


class TestAdd:
    def test_assigns_the_next_unused_identity(self, map_file: Path):
        assert _add(map_file)["FEATURE_ID"] == "F004"

    def test_inserts_at_the_requested_position(self, map_file: Path):
        _add(map_file, after="F001")
        order = [f.identity for f in parse(map_file.read_text(encoding="utf-8"))]
        assert order == ["F000", "F001", "F004", "F002", "F003"]

    def test_added_feature_parses_back_with_its_fields_intact(self, map_file: Path):
        _add(map_file, depends=["F000", "F001"], parallel=True, subfeatures=["a", "b"])
        added = {f.identity: f for f in parse(map_file.read_text(encoding="utf-8"))}["F004"]
        assert added.depends == ["F000", "F001"]
        assert added.parallel
        assert len(added.subfeatures) == 2
        assert added.spec_path is None
        assert not added.done

    def test_rejects_a_dependency_positioned_below_the_insertion_point(self, map_file: Path):
        with pytest.raises(FeatureMapError):
            _add(map_file, after="F001", depends=["F003"])

    def test_rejects_an_invalid_slug(self, map_file: Path):
        with pytest.raises(FeatureMapError):
            _add(map_file, slug="Workspace Indexing")

    def test_rejects_a_feature_with_no_subfeatures(self, map_file: Path):
        with pytest.raises(FeatureMapError):
            _add(map_file, subfeatures=[])

    def test_dry_run_leaves_the_file_untouched(self, map_file: Path):
        before = map_file.read_text(encoding="utf-8")
        assert _add(map_file, dry_run=True)["STATUS"] == "preview"
        assert map_file.read_text(encoding="utf-8") == before

    def test_appending_at_the_end_keeps_the_map_valid(self, map_file: Path):
        _add(map_file, after=None, depends=["F000"])
        features = parse(map_file.read_text(encoding="utf-8"))
        assert features[-1].identity == "F004"
        assert verify(features) == []


class TestRecordSpec:
    def test_writes_the_spec_path_back_into_the_file(self, map_file: Path):
        features = parse(map_file.read_text(encoding="utf-8"))
        feature_map.record_spec(map_file, features, "F001", "specs/002-plugin-host-runtime")
        reparsed = {f.identity: f.spec_path for f in parse(map_file.read_text(encoding="utf-8"))}
        assert reparsed["F001"] == "specs/002-plugin-host-runtime"
        assert reparsed["F000"] == "specs/001-engineering-baseline"


class TestMissingMap:
    def test_default_location_is_under_specs(self):
        # as_posix keeps this assertion true on Windows, where str() renders a backslash.
        assert feature_map.DEFAULT_MAP_RELATIVE.as_posix() == "specs/features-map.md"

    def test_resolve_skips_when_no_map_exists(self, tmp_path: Path, capsys):
        missing = tmp_path / "absent.md"
        assert feature_map.main(["resolve", "--map", str(missing)]) == 0
        assert "SKIPPED" in capsys.readouterr().out

    def test_resolve_does_not_create_a_map(self, tmp_path: Path):
        missing = tmp_path / "absent.md"
        feature_map.main(["resolve", "--map", str(missing)])
        assert not missing.exists()

    def test_verify_does_not_create_a_map(self, tmp_path: Path):
        missing = tmp_path / "absent.md"
        feature_map.main(["verify", "--map", str(missing)])
        assert not missing.exists()

    def test_add_creates_the_map_and_assigns_f000(self, tmp_path: Path, capsys):
        missing = tmp_path / "specs" / "features-map.md"
        code = feature_map.main(
            ["add", "--slug", "engineering-baseline", "--subfeature", "Lint rule", "--map", str(missing)]
        )
        assert code == 0
        assert missing.exists()
        output = capsys.readouterr().out
        assert "CREATED" in output
        features = parse(missing.read_text(encoding="utf-8"))
        assert [f.identity for f in features] == ["F000"]
        assert features[0].slug == "engineering-baseline"
        assert not features[0].done
        assert verify(features) == []

    def test_add_dry_run_does_not_create_the_map(self, tmp_path: Path):
        missing = tmp_path / "specs" / "features-map.md"
        code = feature_map.main(
            ["add", "--slug", "x", "--subfeature", "y", "--map", str(missing), "--dry-run"]
        )
        assert code == 0
        assert not missing.exists()

    def test_second_add_against_a_bootstrapped_map_continues_numbering(self, tmp_path: Path):
        path = tmp_path / "specs" / "features-map.md"
        feature_map.main(["add", "--slug", "first", "--subfeature", "a", "--map", str(path)])
        feature_map.main(
            ["add", "--slug", "second", "--depends", "F000", "--subfeature", "b", "--map", str(path)]
        )
        features = parse(path.read_text(encoding="utf-8"))
        assert [f.identity for f in features] == ["F000", "F001"]
        assert features[1].depends == ["F000"]
        assert verify(features) == []

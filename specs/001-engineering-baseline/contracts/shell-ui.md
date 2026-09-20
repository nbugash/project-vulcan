# Contract: Shell User Interface

**Branch**: `001-engineering-baseline` | **Date**: 2026-09-18

The shell's contract is with the signed-off prototype and with the features that will later
give its controls behaviour. It is a contract because two parties depend on it: the fidelity
gate, which compares it pixel for pixel, and every later feature, which attaches behaviour to
controls this feature creates.

## Regions

Every region below is rendered. An absent region fails SC-009 before it fails anything else.

| Region | Fixed extent | Density-dependent | Interactive here |
| --- | --- | --- | --- |
| Toolbar | 46px height | no | controls respond, no work |
| Remote banner | 26px height | no | visibility follows `remoteBanner` |
| Rail | 44px width | no | selection switches tool window |
| Tool window | — | width: 250 / 276 / 310 | resize, collapse |
| Tab strip | 34px height | no | tab switching |
| Breadcrumbs | 24px height | no | no |
| Editor surface | — | line height, font size | scroll only |
| Dock | — | height: 206 / 236 / 268, capped at 34vh | resize, tab switching |
| Dock header | 30px height | no | collapse |
| Dock tab strip | 28px height | no | tab switching |
| Status bar | 26px height | no | readout variants |
| Command palette | — | no | open, filter, dismiss |
| Run configuration dialog | — | no | open, dismiss |
| Language pack dialog | — | no | open, dismiss |

Extents come from `mockups/assets.md` and are read from the extracted token set rather than
written into layout code, so a prototype change surfaces as a token diff.

## Properties

The six properties the prototype exposes. Each takes effect at runtime without reload.

| Property | Values | Default |
| --- | --- | --- |
| `density` | compact / default / roomy | default |
| `toolSide` | left / right | left |
| `perfReadout` | status / hud / off | status |
| `completionStyle` | list / detail | list |
| `showInlay` | boolean | true |
| `remoteBanner` | boolean | true |

Changing any property re-resolves affected dimensions within the frame budget. A property
change that requires a restart is a defect.

## Behaviour boundary

| Guarantee | Contract |
| --- | --- |
| Controls respond | Every control shows its hover, focus and active treatment |
| Controls do nothing | Every activation routes to the no-op command sink |
| Nothing half-acts | A control MUST NOT perform part of its eventual behaviour |
| No data is real | Content is fixed sample data; no file is read, no process is started, no server is contacted |
| No network | Rendering correctly requires no network access |

The third row is the one later features depend on. A control that partly works cannot be
distinguished from a defect by the feature that inherits it, so partial behaviour is a contract
violation rather than early progress.

## Motion

One animation, `vkpulse`: opacity 0.35 to 1 to 0.35 over 1.1 seconds, used by the streaming
search spinner and the terminal caret. No transitions and no easing curves anywhere else,
matching the prototype and the frame-budget stance.

## Undepicted states

The prototype composes one viewport, one theme, and only the states it draws. Everything else
is a `PrototypeExtension` requiring sign-off before implementation:

- window sizes other than the prototype's viewport
- any second theme
- empty, loading and error states
- hover, focus and disabled treatments the prototype does not show
- overflow behaviour when content exceeds a region

The build fails when the shell can reach a state that has no record.

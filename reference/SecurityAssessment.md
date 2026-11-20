# Security Assessment

The most likely and the most impactful security problems of this software, and how each is handled.
Review this document before each release, together with the [quality checklist](https://github.com/animagram-jp/.github/blob/main/quality_checklist/README.md).

## Scope

| Item | Content |
|-|-|
| Library | The crate `rectgrid` (`src/`). `no_std`, no `unsafe`, no input or output. The only dependency is `libm`. |
| Demo | `examples/`, compiled to wasm32 and served by GitHub Pages. |
| Pipeline | `.github/workflows/`. Publishing to crates.io authenticates through OIDC. |

## Actor

| Actor | Trust | Action |
|-|-|-|
| Calling application | Trusted for the definitions, untrusted for the values. | Defines a unit system (origin and increment functions), then passes coordinates and boundary boxes. |
| End user of the application | Untrusted. | Produces the coordinates (pointer position, drag distance) that the application passes in. |
| Pipeline | Trusted. | Tests, publishes the crate, deploys the demo. |

## Problem

| Problem | Likelihood | Impact | Handling |
|-|-|-|-|
| A coordinate far from the origin makes the scan of an increment function run for a long time and blocks the caller. | Medium | Medium | A non-finite target is rejected with `Error::InvalidInput`. The scan stops at `u32::MAX` segments with `Error::OutOfIndex`. A finite but huge target is still scanned, so an application bounds the coordinates that it accepts from a user. |
| NaN or an infinity enters a boundary test and yields a wrong hit, so that the application acts on the wrong region. | Medium | Medium | Since 0.4.0, NaN is rejected and the `geometry` functions return `Result`. |
| An increment function defined by the application panics or returns a non-finite length. | Low | Low | The definition is reported as `Error::InvalidDefinition` where it cannot be evaluated. The function itself is code of the application. |
| A vulnerable or malicious dependency, or a leaked publishing credential, alters the published crate. | Low | High | The crate depends on `libm` only. No long-lived crates.io token exists, because publishing uses trusted publishing from the `crates-io` environment on a version tag. Actions are pinned by commit SHA. |

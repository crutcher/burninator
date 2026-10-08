# Contributing

burninator cross-tests burn's API across backends: each test records a run on a
reference backend, then checks every enabled target backend against that one
recording.

## Development setup

You need stable Rust (MSRV 1.99.0, from `rust-version` in the workspace
`Cargo.toml`); `rust-toolchain.toml` selects the stable channel, with `rustfmt`
and `clippy`. You also need nightly `rustfmt`: `rustfmt.toml` uses unstable
options, so format with `cargo +nightly fmt`. Stable `rustfmt` ignores those
options and reformats the whole tree.

```sh
rustup toolchain install stable nightly
rustup component add --toolchain nightly rustfmt
```

A GPU backend needs its system stack: the CUDA toolkit for `cuda`, a Vulkan
driver for `vulkan` and `wgpu`.

## Workflow

The IDE's `Test (cpu)` run configuration runs all of these steps: it chains
`Pre-Test` (`Format`, `Fix`, `Clippy`, `Doc Lint`) before the tests. The
configurations are in `.idea/runConfigurations/`. From the shell:

1. Format: `cargo +nightly fmt`
2. Lint: `cargo clippy --workspace --all-targets -- -D warnings`
3. Docs: `cargo doc --no-deps`
4. Test: `cargo test --workspace`
5. Commit.

Warnings are errors: the workspace lints table sets `warnings = "deny"`.

## Backends

A plain `cargo test` runs every cross-test on `flex`, the CPU backend, and
checks it only against its own recording. Each backend feature on
`api-cross-tests` adds that backend to the targets (`testing::target_devices`):

| Feature  | Target device                         |
|----------|---------------------------------------|
| `cpu`    | `Device::cpu()`, CubeCL's CPU backend |
| `cuda`   | `Device::cuda(0)`                     |
| `rocm`   | `Device::rocm(0)`                     |
| `metal`  | `Device::metal(DefaultDevice)`        |
| `vulkan` | `Device::vulkan(DefaultDevice)`       |
| `wgpu`   | `Device::wgpu(DefaultDevice)`         |

Enable any number at once:

```sh
cargo test --workspace --features api-cross-tests/cuda,api-cross-tests/vulkan
```

The `Test (cuda)`, `Test (vulkan)` and `Test (wgpu)` run configurations each
enable one.

## Reference recordings

Cross-tests use `bunsen::audit`. A test's body makes checkpoints; its run on
the reference device, `flex`, is stored as an audit stream, and each target's
run is verified against that stream, checkpoint by checkpoint. A failure names
every target that diverges, and the first checkpoint where it does.

Recordings live under the cargo target directory, out of git:

```text
target/api-cross-tests-reports/flex/{name}.cbor
```

`API_CROSS_TESTS_REPORTS_MODE` says what the reference run does with its
recording:

| Mode             | Recording absent | Recording present   |
|------------------|------------------|---------------------|
| `auto` (unset)   | record           | verify              |
| `record`         | record           | record (replace it) |
| `verify`         | error            | verify              |

Change what a test checks (its inputs, checkpoints or tolerances) and the stored
recording no longer matches; re-record it once:

```sh
API_CROSS_TESTS_REPORTS_MODE=record cargo test --workspace
```

## Adding a cross-test

Put it in the module that mirrors burn's API (`tensor::float::trig::sin` for
`Tensor::sin`), and name its recording by the same path
(`"tensor/float/trig/sin"`). The body is a function
`(&mut AuditProbe<'_>, &Device) -> BunsenResult<()>`; run it with
`testing::audit_targets_against_reference`, as `tensor::float::trig::sin` does.

- Build inputs on the host, so every backend sees the same values; check them
  with an exact checkpoint (`assert_eq_as`).
- Check computed values within a tolerance (`assert_approx_eq_as`): backends do
  not agree bit for bit.
- Keep the checkpoint sequence the same on every backend: events are matched by
  position.

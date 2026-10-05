# Benchmarks

Use `cntryl-stress` for real compiler workloads. Declare each bench target explicitly in
`Cargo.toml` with `harness = false`; benchmark files should define `#[stress]` workloads and end
with `cntryl_stress::stress_main!()`. Add targets when there is a compiler operation to measure.

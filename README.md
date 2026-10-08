# burninator
comparative burn tests

## Example

```terminaloutput
$ cargo test --features cpu,wgpu,vulkan,cuda -- --nocapture
   Compiling api-cross-tests v0.0.0 (/home/crutcher/git/burninator/crates/api-cross-tests)
    Finished `test` profile [optimized + debuginfo] target(s) in 3.05s
     Running unittests src/lib.rs (target/debug/deps/api_cross_tests-aa72b78c8b3c3e7e)

running 1 test
Cross-Test:: tensor/float/trig/sin
- flex: Device<Flex(Cpu)>
- cubecl_cpu: Device<Cube(Cpu(CpuDevice))>
- cubecl_cuda: Device<Cube(Cuda(Cuda(0)))>
- cubecl_wgpu_spirv: Device<Cube(Wgpu(WgpuDevice { kind: DefaultDevice, backend: Vulkan }))>
- cubecl_wgpu_spirv: Device<Cube(Wgpu(WgpuDevice { kind: DefaultDevice, backend: Auto }))>
test tensor::float::trig::sin::tests::test_sin_cos_around_the_circle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.42s

   Doc-tests api_cross_tests

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
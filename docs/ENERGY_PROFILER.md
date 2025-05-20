# Ferra Energy Profiler Design v0.1

> **Status:** Initial Draft - Module 3.2 (Steps 3.2.1 - 3.2.3)

## 1. Introduction and Goals

This document specifies the design for an energy profiling system for the Ferra programming language. The primary aim of this system is to provide developers with insights into the estimated energy consumption of their programs, thereby enabling them to write more energy-efficient software. This is particularly relevant for applications targeting battery-powered devices (mobile, embedded) or for large-scale deployments where energy costs are a significant factor.

The foundational requirement for this profiler, as outlined in `Steps.md` (Item 6: "Security & Energy"), is an **Energy KPI (Key Performance Indicator) based on the formula: µops × TDP → joules**. This implies a model where energy consumption is estimated by counting micro-operations (µops) executed by the program and weighting them by a Thermal Design Power (TDP) characteristic or equivalent power/energy factor. The initial implementation target for this estimation is an LLVM pass or an equivalent mechanism.

*   **Purpose**:
    *   To define a methodology and tooling for estimating the energy footprint of Ferra programs.
    *   To empower developers to make informed decisions about code structures and algorithms based on their potential energy impact.
    *   To integrate energy awareness into the Ferra development lifecycle, including testing and CI.

*   **Core Requirement (from `Steps.md`)**:
    *   **Energy KPI Formula**: LLVM pass (or equivalent) counts µops × TDP → joules.
    *   **CI Metric**: CI fails if > 70 J (this target is primarily for the test suite integration as a starting point).

*   **Goals for this Design Document**:
    1.  **Specify Energy Estimation Mechanism (Step 3.2.1)**: Detail how µops will be defined and counted, how TDP values (or their equivalents) will be modeled and applied, and how the LLVM pass (or an alternative) will perform this estimation.
    2.  **Specify Test Suite Integration (Step 3.2.2)**: Define how the energy profiler will be integrated with Ferra's test suite, including how the < 70 J target (or other relevant budgets) will be applied and reported for tests.
    3.  **Design CI Checks (Step 3.2.3)**: Outline how Continuous Integration (CI) pipelines will automatically run the energy profiler and enforce defined energy budgets, flagging regressions or excessive consumption.
    4.  **Outline Data Sources and Reporting**: Discuss potential sources for µop costs and TDP data, how profiling results will be reported to the user, and any necessary tooling or configuration options.
    5.  **Acknowledge Model Limitations**: Clearly state the assumptions and limitations of the proposed energy estimation model.

This document aims to provide a clear specification for building the first version of the Ferra Energy Profiler, focusing on the core requirements and establishing a foundation for future enhancements.

## 2. Energy Estimation Mechanism (Step 3.2.1)

### 2.1. Core Formula: µops × TDP → Joules

The foundational principle for energy estimation in Ferra, as mandated by `Steps.md` (Item 6), is the formula:

**Estimated Energy (Joules) = Σ (µops_i × TDP_factor_i)**

Where:
*   **µops_i**: Represents a count of micro-operations (or a weighted equivalent) for a given instruction or operation type `i`. This is a proxy for the computational work done.
*   **TDP_factor_i**: Represents a factor related to the Thermal Design Power (or a more granular energy cost) associated with executing µop type `i` on a target architecture. This serves as a proxy for the power or energy intensity of that operation.
*   The summation (Σ) occurs over all relevant operations executed by the program or code segment being profiled.

**High-Level Explanation of Components**:

*   **Micro-operations (µops)**: This term generally refers to simple, elementary operations that a CPU might break complex instructions into. For our model, "µops" will be an abstracted unit of computational work. The precise definition and how these are counted or weighted for different Ferra IR or LLVM IR instructions is detailed in Section 2.2. The goal is to assign a "µop cost" to each instruction.
*   **TDP (Thermal Design Power) Factor**: TDP is traditionally a measure of the maximum heat a CPU is expected to generate under normal operating conditions, often used as an indicator of power consumption. In our model, "TDP_factor" will be a coefficient or set of coefficients that translate µop counts into estimated energy. This might be a simplified, architecture-average TDP value, or a more nuanced model with different factors for different classes of µops or CPU functional units. This is detailed in Section 2.3.
*   **Joules**: The unit of energy. The formula aims to provide an *estimation* of energy consumed, not a precise physical measurement.

This formula provides a simplified, model-based approach to energy estimation that can be implemented within a compiler pass without requiring direct hardware power measurement during compilation or typical program execution. The accuracy of the estimation will depend heavily on the quality of the µop counting and the TDP model.

### 2.2. Defining and Counting µops (Micro-operations)

The concept of "µops" (micro-operations) is central to the energy estimation formula. For the Ferra energy profiler, µops will serve as an abstract representation of computational work. This section details how these µops are defined, at what level of program representation they are counted, and how they are weighted.

*   **Level of Abstraction for µop Counting**:
    *   **Primary Approach: LLVM IR Instruction Level**:
        *   The most practical initial approach is to count µops (or assign µop weights) based on LLVM IR instructions. This is because:
            *   LLVM IR is a well-defined, architecture-agnostic (to a large extent) representation before machine-specific code generation.
            *   LLVM passes operate directly on this IR, making it feasible to instrument or analyze.
            *   Many LLVM IR instructions have a relatively consistent level of complexity, though some (like calls or memory operations) are more variable.
        *   This aligns with the "LLVM pass" requirement from `Steps.md`.
    *   **Secondary/Alternative: Ferra IR Instruction Level (Consideration)**:
        *   Counting at the Ferra IR level (see `IR_SPECIFICATION.md`) could provide earlier insights but might be less representative of final machine work due to subsequent optimizations and lowering to LLVM IR.
        *   Could be a supplementary source or used for very high-level estimates if an LLVM pass is initially too complex.
    *   **Machine Instruction Level (Out of Scope for v0.1 Model)**:
        *   Counting actual machine instructions or CPU-decoded µops is highly architecture-specific and complex to implement portably within a compiler pass. While most accurate for a specific CPU, this is beyond the scope of the initial model, which aims for a more generalized estimation.

*   **Assigning µop Weights to Instructions**:
    *   **Not all instructions are equal**: A simple integer addition is less work (and consumes less energy) than a floating-point division or a memory load/store that might miss cache.
    *   **Weighting Strategy**: Instead of a simple 1:1 count (1 instruction = 1 µop), each relevant LLVM IR instruction (or instruction category) will be assigned a "µop weight".
        *   **Example Weights (Illustrative - TBD EP-UOP-1)**:
            *   Integer ALU (add, sub, and, or): 1 µop_unit
            *   Integer MUL/DIV: 3-5 µop_units
            *   Float ALU (fadd, fsub): 2 µop_units
            *   Float MUL/DIV: 4-8 µop_units
            *   Memory Load/Store: 5-10 µop_units (highly variable due to cache, could be a base cost)
            *   Branch/Jump: 1-2 µop_units
            *   Function Call Overhead: X µop_units (plus cost of callee)
            *   Intrinsics/Special Ops: Specific weights based on their complexity.
    *   **Source of Weights**: These weights would initially be based on:
        *   Published instruction latencies/throughputs for generic modern CPU architectures.
        *   Academic research on instruction-level power modeling.
        *   Heuristics and educated estimates.
        *   (Future) Calibration against actual hardware measurements (Section 2.6).
    *   **A "µop_unit"**: This would be an abstract unit. The TDP factor (Section 2.3) would then convert these abstract µop_units into estimated energy.

*   **Strategy for Assigning Weights/Counts**:
    *   A table or database mapping LLVM IR opcodes (and potentially operand types/attributes) to their µop weights will be maintained.
    *   The LLVM pass will query this table for each instruction it processes.
    *   **Default Weight**: Instructions not explicitly in the table might receive a default weight or be ignored (TBD).

*   **Consideration of Dynamic Factors vs. Static Analysis**:
    *   **Static Analysis (Primary for v0.1)**:
        *   The initial LLVM pass will likely perform a static analysis, counting the µop weights of instructions as they appear in the IR.
        *   **Loop Iterations**: For loops, a simple heuristic might be used (e.g., assume N iterations, or use profile data if available in the future). Unbounded loops are problematic for static estimation. (This is a key area for TBD EP-UOP-1).
        *   **Branch Probabilities**: For conditional branches, the pass might initially assume equal probability (50/50) or sum costs for all paths if a worst-case or average static estimate is desired. More advanced static branch prediction heuristics could be applied.
    *   **Dynamic Factors (Future/Advanced)**:
        *   True dynamic µop counts require execution profiling (e.g., instrumented builds or hardware performance counters), which is more complex than the static LLVM pass model specified for v0.1.
        *   The current model focuses on a *static estimation* of energy cost based on the compiled code's structure, not a dynamic measurement of a specific execution run. This makes it suitable for CI checks on committed code.
        *   Profile-Guided Optimization (PGO) data, if integrated into Ferra (Module 4.2), could later inform more accurate static estimations of loop counts and branch probabilities.

The goal for v0.1 is to establish a reasonable static estimation model based on weighted LLVM IR instructions. The precise weights and handling of control flow will be critical and are marked as TBD EP-UOP-1.

### 2.3. TDP (Thermal Design Power) Model

The "TDP_factor" in our energy estimation formula (`µops × TDP_factor → Joules`) is critical for converting abstract µop counts into an estimated energy value. This section discusses how TDP will be modeled and sourced for the Ferra energy profiler.

*   **Definition and Sourcing of TDP Information**:
    *   **Traditional TDP**: CPU manufacturers often specify a Thermal Design Power (TDP) value in watts. This typically represents the maximum amount of heat a CPU is expected to generate under a typical heavy workload, not necessarily its absolute peak power consumption or its power consumption for a specific instruction.
    *   **Our "TDP_factor"**: For the profiler, the `TDP_factor` will not be a direct, single CPU TDP value. Instead, it will be a coefficient or a set of coefficients that represent an *average energy cost per weighted µop_unit* for a target class of architectures.
    *   **Initial Sourcing (TBD EP-TDP-1)**:
        *   **Generalized Architecture Profiles**: For v0.1, we will likely start with generalized TDP-related factors for broad architecture classes (e.g., "generic x86-64 desktop," "generic ARM mobile"). These factors would be derived from:
            *   Publicly available data on average power consumption of CPU classes.
            *   Academic research on instruction-level power models.
            *   Potentially, a simplified energy-per-instruction-type model if µop weights are sufficiently granular.
        *   **Configurable Values**: It should be possible for the user or build system to provide or select a TDP profile or a global TDP-related scaling factor if more specific information is available for their target hardware.
        *   **Per-CPU Model (Future)**: A more advanced system might allow for specific TDP profiles based on CPU models (e.g., "Intel Core i7-13700K," "Apple M2"), but this requires a significant database and is out of scope for v0.1.

*   **Granularity of the TDP_factor**:
    *   **Option 1: Single Scaler (Simplest)**: A single `TDP_factor` (e.g., joules per µop_unit) could be used. The total µop_units for the program would be multiplied by this factor. This is the simplest to implement but least accurate.
    *   **Option 2: Weighted Factors per µop Category (More Realistic)**:
        *   Different types of operations (ALU, memory access, floating point) consume different amounts of power even if their abstract "µop weights" (from Section 2.2) were normalized.
        *   The `TDP_factor` could actually be a set of energy coefficients, one for each category of µop_unit (e.g., `TDP_factor_ALU`, `TDP_factor_MEM`, `TDP_factor_FP`).
        *   The formula would then be: `Σ (µops_ALU × TDP_factor_ALU) + Σ (µops_MEM × TDP_factor_MEM) + ...`
        *   This approach is more complex but potentially more representative. The v0.1 design should aim for this if feasible, with details TBD EP-TDP-1.
    *   **Not Per-Instruction TDP**: Assigning a unique TDP value to every single CPU instruction is too granular and complex for this model. The abstraction happens at the µop_unit and TDP_factor_category level.

*   **Relationship to Actual Power Draw and Model Limitations**:
    *   **Estimation, Not Measurement**: It is crucial to emphasize that this model provides an *estimation* of energy consumption, not a direct physical measurement.
    *   **Factors Not Modeled**: Many factors influence actual CPU power draw that are difficult to model statically:
        *   Specific CPU microarchitecture and manufacturing variations.
        *   Dynamic voltage and frequency scaling (DVFS).
        *   Cache performance (hits/misses).
        *   Memory bandwidth utilization.
        *   Power states of other system components (GPU, RAM, peripherals).
        *   Operating system activity and background processes.
        *   Ambient temperature and cooling solutions.
    *   **Purpose of the Model**: The goal is not perfect accuracy but to provide a *relative* measure for comparing the energy efficiency of different code versions or algorithms and to flag significant energy regressions within the defined model. It's a tool for *energy-aware development guidance*.

The precise definition of TDP factors, their granularity, and their default values for different architectural profiles are key details to be resolved (TBD EP-TDP-1). For v0.1, a pragmatic approach with a few configurable, generalized architectural profiles is envisioned.

### 2.4. Implementation Strategy: LLVM Pass (Primary)

The primary mechanism for implementing the energy estimation model will be an LLVM pass. This aligns with the directive in `Steps.md` and leverages LLVM's infrastructure for code analysis and transformation. This pass will operate on LLVM Intermediate Representation (IR).

*   **Pass Type and Placement**:
    *   The energy profiler pass will likely be an **Analysis Pass**. It analyzes the code to gather information (µop counts) but does not transform the IR itself.
    *   It should run relatively late in the pass pipeline, after most major optimizations have occurred, so the µop counts reflect code that is closer to what will be executed. However, it must run before final machine code generation. The exact placement needs careful consideration (TBD EP-PASS-IMPL-1).
    *   (Ref: `BACKEND_LLVM_X86-64.md` Section 5 for LLVM optimization passes).

*   **Traversal and Analysis**:
    *   The pass will iterate over all functions in an LLVM module.
    *   Within each function, it will iterate over basic blocks and then over individual LLVM IR instructions.

*   **Identifying and Counting µops**:
    *   For each LLVM IR instruction encountered, the pass will:
        1.  Identify the instruction's opcode (e.g., `add`, `load`, `fmul`).
        2.  Consult the µop weight table (defined in Section 2.2 strategy) to get the µop_unit cost for that opcode (and potentially its operands' types or other attributes).
        3.  Accumulate these weighted µop_units.
    *   **Handling Control Flow**:
        *   **Static Counts**: As discussed in Section 2.2, for loops and branches, the initial v0.1 pass will use static heuristics.
            *   Loops: A configurable assumed iteration count (e.g., N=10 for a generic loop, or a specific value if loop metadata is available) or a simple summation of one iteration's cost. Unbounded loops will be flagged or assigned a high default cost.
            *   Branches: Potentially sum the costs of µops on the most expensive path, or average if probabilities can be estimated (e.g., 50/50 for simple `if`).
        *   This static approach means the reported energy is for a *characteristic execution path* or a *potential worst-case/average static cost* rather than a specific dynamic run.

*   **Applying TDP Weights/Factors**:
    *   Once the total weighted µop_units are accumulated for a function or module, the TDP_factor(s) (from Section 2.3 model) are applied to convert this abstract work unit into an estimated energy value in Joules.
    *   If TDP_factors are per µop category (e.g., ALU, Memory), the pass will need to accumulate µop_units per category and then apply the respective TDP_factors before summing.

*   **Interface and Output of the Pass**:
    *   **Invocation**: The pass would be invoked as part of the Ferra compilation process when energy profiling is enabled (e.g., via a CLI flag, see Section 5.1).
    *   **Data Output**:
        *   The primary output will be the total estimated energy (Joules) for the compiled module.
        *   It might also output (optionally, for verbose modes):
            *   Energy estimates per function.
            *   Total weighted µop_units (and per category if applicable).
        *   The format for this data could be simple text to stdout/stderr, or a structured format like JSON for easier parsing by other tools (e.g., test runners, CI scripts). (See Section 5.3).
    *   **Metadata/Annotations (Internal)**: The pass might internally annotate LLVM IR with energy information for debugging or consumption by subsequent custom passes, but this is not the primary output for the user/CI.

*   **Interaction with Existing LLVM Optimization Passes**:
    *   **Placement**: As mentioned, running after major optimizations is key. If run too early, the µop counts might not reflect the code that actually gets executed.
    *   **Impact of Optimizations**: Optimizations like inlining, loop unrolling, instruction combining, and dead code elimination will significantly affect the LLVM IR seen by the energy pass. This is desirable, as the energy estimate should reflect the optimized code.
    *   **No Transformation**: The energy profiler pass itself should not modify the IR in a way that affects correctness or other optimizations; it's an analysis and reporting pass.

*   **Referencing `BACKEND_LLVM_X86-64.md`**:
    *   The LLVM pass will operate within the context of the LLVM backend described in `BACKEND_LLVM_X86-64.md`. It will use the LLVM C++ APIs for IR traversal and analysis.
    *   Knowledge of Ferra's type mapping to LLVM IR (Section 3.1 of `BACKEND_LLVM_X86-64.md`) will be essential for correctly interpreting instructions and potentially refining µop weights based on operand types.

This LLVM pass-based strategy provides a direct path to analyze code close to its final form while leveraging LLVM's extensive infrastructure. The main challenges will be defining robust µop weights and heuristics for static control flow analysis.

### 2.5. Alternative/Equivalent Mechanisms (If LLVM Pass is not sole approach)

While an LLVM pass is the primary envisioned strategy due to its ability to analyze optimized IR, other mechanisms could complement it or serve as alternatives, especially for different stages of development or levels of detail. The phrase "or equivalent" in `Steps.md` allows for this flexibility.

*   **Analysis at Ferra IR Level**:
    *   **Pros**:
        *   Can provide energy estimates earlier in the compilation pipeline, before LLVM-specific optimizations.
        *   Operates on a higher-level IR (see `IR_SPECIFICATION.md`) that might be simpler to assign initial µop weights to, as it's closer to Ferra source semantics.
        *   Potentially faster to implement a basic estimator than a full LLVM pass.
    *   **Cons**:
        *   Less accurate, as it doesn't account for LLVM optimizations which can significantly change instruction counts and types.
        *   Might miss opportunities for architecture-specific µop considerations that LLVM IR exposes.
    *   **Potential Use**: Could be used for rapid feedback during development or for very coarse-grained "hotspot" identification before full optimization. It would use a separate µop weight table tailored to Ferra IR opcodes.

*   **Static Analysis of Emitted Assembly or Machine Code**:
    *   **Pros**:
        *   Analyzes the code closest to what will actually execute on the hardware.
        *   Can account for machine-specific instruction selection and scheduling to some extent.
    *   **Cons**:
        *   Highly architecture-dependent (e.g., different assembly for x86-64 vs. ARM64).
        *   Parsing and analyzing assembly/machine code is significantly more complex than IR.
        *   Mapping back to source code or even LLVM IR for reporting can be challenging.
        *   Attributing TDP factors at this level can be very difficult without detailed microarchitectural power models.
    *   **Potential Use**: More of a research direction or for highly specialized, target-specific tuning. Unlikely for the general v0.1 profiler.

*   **Combination of Approaches**:
    *   A hybrid model could be considered in the future. For example:
        *   Initial estimation at Ferra IR level for quick developer feedback.
        *   More detailed estimation via the LLVM pass for CI checks and release profiling.
        *   (Very Future) Use machine code analysis or hardware performance counter data (dynamic analysis) to calibrate the static models.

*   **Leveraging Semantic Tags (`IR_SEMANTIC_TAGS.md`)**:
    *   If certain Ferra IR operations have known high energy costs not easily visible in LLVM IR, semantic tags could annotate these. The LLVM pass could then look for these tags (passed through as metadata) and adjust its µop counts accordingly.
    *   Tags could also guide heuristics for loop counts or branch probabilities if populated by other analysis phases.

For v0.1, the focus remains on the LLVM pass as the primary mechanism for delivering on the core requirement. These alternatives are noted for completeness and future exploration if the LLVM pass proves insufficient or too narrow in its applicability for all desired use cases.

### 2.6. Data Sources and Calibration (Future Consideration)

The accuracy of the `µops × TDP_factor` energy estimation model heavily depends on the quality of the µop weights and TDP factors used. While v0.1 will start with heuristics and publicly available data, future enhancements should focus on more empirical data sources and calibration.

*   **Potential for Calibrating µop Weights and TDP Models**:
    *   **Against Real Hardware Measurements**:
        *   Run a suite of microbenchmarks (each exercising specific instruction types or code patterns) on actual target hardware equipped with power measurement capabilities (e.g., Intel RAPL, specialized power measurement tools).
        *   Correlate the measured energy consumption of these microbenchmarks with the µop counts generated by the Ferra energy profiler for the same benchmarks.
        *   Use statistical methods to refine µop weights and TDP factors to better fit the observed data.
        *   This is a significant research and engineering effort.
    *   **Iterative Refinement**: The models should be designed to be updatable as more calibration data becomes available or as new architectures emerge.

*   **Sources for CPU µop Counts or Power Characteristics**:
    *   **Manufacturer Documentation**: CPU vendors (Intel, AMD, ARM, etc.) sometimes publish detailed microarchitectural guides, optimization manuals, or instruction tables that may include information on µop decomposition, port utilization, or even relative instruction costs.
    *   **Academic Research Papers**: Many papers in computer architecture and compiler design focus on instruction-level power modeling, performance analysis, and energy characterization. These can provide methodologies and data for various CPU architectures.
    *   **Performance Counter Analysis**: Hardware Performance Monitoring Units (PMUs) in modern CPUs expose counters for various events (e.g., instructions retired, cycles, cache misses, branch mispredictions, and sometimes even more specific µop-related events). Analyzing these for specific code sections can inform µop weighting.
    *   **Tools like `llvm-mca` (LLVM Machine Code Analyzer)**: While `llvm-mca` primarily analyzes throughput and latency, its underlying machine-scheduler models might provide insights into the relative complexity (and thus potential energy cost) of different instruction sequences for supported targets.

*   **Database of µop Weights and TDP Profiles**:
    *   As the system matures, a database of µop weights (for LLVM IR opcodes) and TDP profiles (for different architectural classes or even specific CPU models) could be developed and maintained.
    *   This database would be a key asset for the energy profiler and would need a clear process for updates and versioning.

For the v0.1 profiler, the approach will be to start with a reasonable, well-documented set of initial µop weights and generalized TDP factors. The infrastructure should be designed to allow these to be updated or overridden as more sophisticated data becomes available or as calibration efforts are undertaken in the future. The focus is on relative accuracy and trend identification initially.

## 3. Test Suite Integration (Step 3.2.2)

Integrating the energy profiler into Ferra's test suite is crucial for establishing baseline energy consumption metrics, tracking regressions, and encouraging energy-aware development practices from the outset.

### 3.1. Target Energy Budget: < 70 J (from `Steps.md`)

`Steps.md` (Item 6) specifies a metric: "CI fails > 70 J". While this is primarily a CI enforcement rule (covered in Section 4), it also informs the target for test suite integration. The example given in `Steps.md` (Item 12) states: "Compilation on an M2-Air (reference laptop) finishes in **1.8 s**, energy 56 J, binary size 1.3 MB ↓-strip" for a `greet.lang` example.

*   **Interpretation of the 70 J Target for Tests**:
    *   **Initial Focus: Reference Test Case(s)**: The 70 J target likely serves as an initial benchmark for a specific, well-defined reference test case or a small suite of core functionality tests, rather than an absolute limit for *every single* unit test. Many unit tests will be too small to register significant energy consumption individually with the proposed model.
    *   **Not a Per-Unit-Test Cap (Initially)**: Applying a strict < 70 J cap to every individual unit test might be impractical, as some tests might legitimately involve more complex computations.
    *   **Reference Suite Budget**: The most practical interpretation for v0.1 test suite integration is to define a specific "energy benchmark suite" (a subset of the full test suite, or dedicated energy tests) whose total estimated energy consumption should remain below a defined budget, with 70 J serving as an initial example for such a suite or a key test therein.
    *   **Example Context (greet.lang)**: The 56 J for `greet.lang` compilation (as per `Steps.md`) provides a concrete data point. This implies that compiling and perhaps running a simple program should fall well within this kind of budget. Our test suite should include tests that model similar or slightly more complex scenarios to track against this.

*   **Informing the Target**:
    *   The 70 J figure acts as an **aspirational target and a calibration point**. As the energy profiler model (µop weights, TDP factors) is refined, this target might be adjusted, or more granular targets for different test categories might be introduced.
    *   The goal is to have specific tests or test suites whose energy "score" can be consistently tracked over time. An increase in this score would indicate a potential energy regression.

*   **Defining "Test" for Energy Budgeting**:
    *   For the purpose of energy budgeting in tests, a "test" could be:
        1.  The compilation of a specific Ferra source file or small project.
        2.  The execution of a compiled Ferra program (if the profiler can estimate runtime energy based on static analysis of execution paths, though this is more advanced).
        3.  For v0.1, the focus will likely be on the **energy cost of compiling representative code samples** or the **estimated static energy cost of the compiled artifacts themselves**, as dynamic runtime energy profiling is more complex.

The precise definition of which tests fall under this budget and how the budget is aggregated (e.g., sum of specific tests, average) will be part of the test suite design (TBD EP-TARGET-1).

### 3.2. Running the Profiler on Tests

To integrate energy estimation into the testing workflow, the Ferra test runner or build system needs a mechanism to invoke the energy profiler for relevant test cases and collect its output.

*   **Invocation During Test Execution**:
    *   **Compiler Flag**: The primary method for enabling energy profiling for a test will be via a compiler flag (e.g., `--enable-energy-profile`, see Section 5.1).
    *   **Test Runner Integration**: The Ferra test runner (or the build system executing tests) will be responsible for:
        1.  Identifying tests that are designated as "energy benchmark tests" or part of an "energy suite." This could be through naming conventions, specific test attributes/annotations (e.g., `#[energy_test(budget=70J)]`), or dedicated test suite configurations.
        2.  For these tests, invoking the Ferra compiler with the energy profiling flag enabled when building the test executable or the code under test.
        3.  If the profiler estimates the energy cost of *compilation*, the test runner captures this output directly from the compilation step.
        4.  If the profiler estimates the *static energy cost of the compiled artifact*, the test runner would first compile the code (with profiling enabled) and then potentially run a separate tool or analyze the profiler's output associated with that artifact.
    *   **Granularity**: Profiling might be enabled for:
        *   The compilation of the test executable itself.
        *   The compilation of specific library code that the test exercises.

*   **Mechanism for Collecting Energy Data**:
    *   **Profiler Output**: The energy profiler (LLVM pass or equivalent) will output the estimated energy data, likely to stdout/stderr when the specific compilation unit is processed, or to a designated file (see Section 5.3).
    *   **Test Harness Responsibility**: The test harness or test execution script will:
        *   Capture this output.
        *   Parse it to extract the relevant energy metrics (e.g., total Joules, µop counts).
        *   Associate these metrics with the specific test case or test suite being run.
    *   **Data Storage (Temporary)**: During a test run, this data might be stored temporarily before being aggregated or reported.
    *   **Avoiding Interference**: The act of profiling should ideally not significantly interfere with the performance or behavior of the tests themselves, though the compilation step might be slightly slower if the profiler adds overhead.

*   **Types of Tests for Energy Profiling**:
    *   **Compilation Energy Tests**: Tests that measure the energy cost of compiling specific, representative Ferra code samples.
    *   **Static Artifact Energy Tests**: Tests that compile a piece of code and then analyze the static energy estimate of the *resulting artifact*. This is the most likely focus for v0.1, as it directly reflects the code's intrinsic properties.
    *   **Runtime Energy Estimation (Future)**: While true dynamic runtime energy profiling is complex, future static analysis might try to estimate runtime energy by analyzing control flow graphs of compiled code and applying execution count heuristics. Tests could then be designed to exercise specific execution paths.

The goal is to make energy profiling a configurable part of the test execution process for designated benchmarks, allowing for consistent data collection.

### 3.3. Reporting Energy Consumption in Test Results

Clear reporting of energy consumption metrics within test results is essential for developer visibility and for tracking performance over time.

*   **Format for Reporting**:
    *   **Standard Test Output**: For quick visibility, key energy metrics (e.g., estimated Joules) for profiled tests should be included in the standard console output of the test runner. This could be similar to how test duration is often reported.
        *   Example: `test my_energy_benchmark ... ok (150ms, 65.3 J)`
    *   **Structured Log Files**: For more detailed analysis and for CI consumption, energy profiling results should also be logged in a structured format (e.g., JSON, XML, CSV - see Section 5.3 for profiler output format). This allows for easier parsing and integration with other tools.
        *   Each test report could include an "energy_profile" section with detailed metrics.
    *   **Human-Readable Summary**: The test runner might also produce a summary report at the end of a test run, highlighting any energy benchmarks that exceeded their budgets or showed significant regressions.

*   **Granularity of Reporting**:
    *   **Per-Test**: At a minimum, energy metrics should be reported for each individual test designated as an energy benchmark.
    *   **Per-Module/Suite (Optional)**: If tests are organized into suites or modules, an aggregated energy score for the suite could also be reported.
    *   **Breakdown (Future)**: More advanced reporting might include a breakdown of energy consumption by function or code region within a profiled test, if the profiler supports this level of detail (see Section 7, Future Work). For v0.1, a total for the profiled compilation unit/artifact is the primary goal.

*   **Consistency with `DESIGN_DIAGNOSTICS.md`**:
    *   If energy budget violations are treated as a form of test failure or warning, the messages should align with the principles (e.g., "positive-first") and formatting guidelines in **DESIGN_DIAGNOSTICS.md**.
    *   Any structured diagnostic output (e.g., JSON from test failures related to energy) should aim to use or be compatible with the core diagnostic fields defined in **DESIGN_DIAGNOSTICS.md** (e.g., `code`, `severity`, `message`, `file`, `span` if applicable).
    *   Diagnostic codes for energy budget issues in tests could be considered (e.g., `ET_XXX` for Energy Test, as registered in `diagnostic_codes.md`).

### 3.4. Enforcing the Budget in Local Test Runs

While CI will perform strict budget enforcement (Section 4), developers should also be able to check for and be alerted to budget violations in their local development environment.

*   **Mechanism for Local Enforcement**:
    *   **Test Runner Logic**: The Ferra test runner, when executing energy-profiled tests, will compare the reported energy value against a predefined budget for that test or test suite.
    *   **Budget Definition**: Budgets could be defined via:
        *   Test attributes/annotations: `#[energy_test(budget=70.0)]` (as conceptualized in 3.2).
        *   Configuration files for test suites.
        *   Command-line arguments to the test runner.
*   **Alerting Developers**:
    *   **Test Failure**: If a test exceeds its defined energy budget, it should be marked as a **FAIL** in the test run output. The failure message should clearly indicate the budget, the actual estimated energy, and the amount by which it was exceeded.
        *   Example: `FAIL: my_energy_benchmark - Energy budget exceeded: expected <= 70.0 J, actual = 75.2 J (+5.2 J)`
    *   **Warnings for "Near Budget"**: Optionally, the test runner could issue warnings if a test is approaching its budget (e.g., within 5-10%), to alert developers to potential future issues.
    *   **Verbose Output**: Developers should be able to run tests with verbose output that shows the detailed energy metrics for all profiled tests, not just failures.

*   **Configurability**:
    *   It should be possible to temporarily disable strict budget enforcement locally (e.g., via a test runner flag like `--no-energy-fail`) to allow developers to iterate on code that might temporarily exceed budgets, without blocking their local test runs. CI would still enforce it strictly.
    *   The default behavior should be to enforce the budgets specified.

This local feedback loop is crucial for enabling developers to proactively manage the energy footprint of their code before it reaches CI.

## 4. CI Checks for Energy Budget (Step 3.2.3)

Continuous Integration (CI) plays a vital role in automatically enforcing quality standards, including energy efficiency. This section outlines how the Ferra energy profiler will be integrated into the CI workflow to monitor and enforce energy budgets.

### 4.1. CI Workflow Integration

The energy profiler needs to be seamlessly integrated into the project's CI pipeline to provide automated checks.

*   **Invocation in CI**:
    *   The CI pipeline (e.g., GitHub Actions, GitLab CI, Jenkins) will be configured to execute the Ferra test suite, including the designated energy benchmark tests.
    *   During these test runs, the test runner will invoke the Ferra compiler with energy profiling enabled (e.g., via the `--enable-energy-profile` flag or an equivalent environment variable set by the CI job).
    *   The CI script will need to ensure that any necessary TDP profile data or µop weight configurations (if applicable beyond defaults) are available in the CI environment.

*   **Trigger Conditions**:
    *   **Per Pull Request/Merge Request**: Energy checks should ideally run for every PR/MR targeting the main development branches. This provides early feedback on the energy impact of proposed changes.
    *   **Nightly/Scheduled Builds**: Comprehensive energy benchmark suites, which might be more time-consuming, could run on a nightly or scheduled basis against the latest development branch to track trends and catch slower-emerging regressions.
    *   **Release Builds**: Before tagging a release, a full energy profile run should be mandatory to ensure the release candidate meets energy targets.

*   **Environment Consistency**:
    *   To ensure consistent and comparable energy estimates, CI jobs running energy profiles should execute in a standardized environment (e.g., specific VM type, container image) as much as possible. While the model is static, compiler behavior or library availability could subtly influence results.
    *   The "reference" architecture profile for TDP/µop weights used in CI should be well-defined and version-controlled.

### 4.2. Budget Enforcement and Failure Conditions

The primary role of CI integration is to automatically enforce the defined energy budgets.

*   **Using the < 70 J Target (and other budgets)**:
    *   As discussed in Section 3.1, the "< 70 J" target from `Steps.md` will initially apply to specific reference test cases or a defined "energy benchmark suite."
    *   The CI system will compare the profiler's output for these designated tests/suites against their configured budgets.
    *   More granular budgets might be defined for different modules or critical code paths as the system matures.

*   **Defining a CI Failure**:
    *   A CI job step associated with energy profiling will **fail** if:
        1.  Any designated energy benchmark test or suite exceeds its predefined energy budget (e.g., reported Joules > budget Joules).
        2.  A significant, unexplained regression in energy score is detected compared to a baseline (e.g., previous successful build on the main branch), even if still within an absolute budget (this is a more advanced check for future consideration).
    *   The failure should clearly indicate which test/suite failed and the budget vs. actual values.

*   **Strictness of Enforcement**:
    *   For PRs/MRs, an energy budget violation should ideally block the merge until addressed or explicitly overridden with justification (e.g., if the increased energy cost is deemed acceptable for new functionality).
    *   For nightly/main branch builds, failures should alert the development team immediately.

*   **Baseline Management (Future)**:
    *   For regression tracking, the CI system could store energy metrics from successful main branch builds as a baseline. Subsequent PRs could be compared against this baseline to detect relative increases in energy consumption.

### 4.3. Reporting in CI

Clear and accessible reporting of energy metrics in the CI system is crucial for developers to understand and address issues.

*   **CI Log Output**:
    *   The test runner output (including energy metrics and budget violations, as per Section 3.3) will be visible in the CI job logs.
    *   Failures should be clearly highlighted with error messages, adhering to the diagnostic principles from **DESIGN_DIAGNOSTICS.md**.

*   **Structured Artifacts**:
    *   The CI job should archive the structured log files (e.g., JSON, as per Section 5.3) containing detailed energy profiling results. This allows for offline analysis and can be consumed by dashboards or other reporting tools. The structure should be compatible with or extend the schema in **DESIGN_DIAGNOSTICS.md** where applicable (e.g., if reporting specific errors or warnings from the profiler itself).

*   **Dashboarding / Visualization (Future Consideration)**:
    *   Ideally, energy consumption metrics could be integrated into a project dashboard.
    *   This could show:
        *   Current energy scores for key benchmarks.
        *   Trends over time for these benchmarks.
        *   Visual indication of budget adherence.
        *   Comparisons between branches or PRs.
    *   Tools like Grafana, or custom web UIs, could consume the structured log artifacts.

*   **Pull Request/Merge Request Comments**:
    *   For CI systems that support it (e.g., GitHub Actions bots), a summary of energy check results (pass/fail, key metrics, link to detailed logs) could be automatically posted as a comment on the PR/MR. This provides immediate visibility to reviewers and authors.

*   **CI Status Badge/Indicator Example (Conceptual)**:
    *   Projects might include a status badge in their `README.md` indicating the energy compliance of the main branch for key benchmarks.
        *   Example Markdown for a badge: `![Energy Benchmark](https://my_ci_server.com/api/projects/ferra/badges/main/energy_status.svg?benchmark=core_suite)`
    *   A simple CI script step that checks the energy results and prints a status could be (illustrative):
        ```sh
        # Assume energy_results.json contains {"benchmark_name": "core_suite", "joules": 65.0, "budget": 70.0}
        joules=$(jq .joules energy_results.json)
        budget=$(jq .budget energy_results.json)
        if (( $(echo "$joules <= $budget" | bc -l) )); then
          echo "Energy Check (core_suite): PASS - $joules J (Budget: $budget J)"
        else
          echo "Energy Check (core_suite): FAIL - $joules J (Budget: $budget J)"
          # exit 1 # to fail CI job
        fi
        ```

*   **Notifications**:
    *   Failures in energy budget checks on main branches should trigger notifications to the development team (e.g., via email, Slack, or other integrated communication channels).

By integrating these checks and reporting mechanisms, the CI system becomes a key enforcer of Ferra's energy efficiency goals.

## 5. Tooling and Configuration

To make the energy profiler usable and adaptable, appropriate tooling and configuration options must be provided. This primarily involves compiler command-line interface (CLI) options and considerations for managing any necessary profile data.

### 5.1. Compiler CLI Options

The Ferra compiler CLI (e.g., `ferra build`, `ferra test`) will be extended with options to control energy profiling. These options should integrate naturally with the existing CLI structure (see `BACKEND_LLVM_X86-64.md` Section 4.3 for examples of current backend options).

*   **Conceptual EBNF for Energy Profiler CLI Options** (illustrative, actual parsing depends on CLI framework):
    ```ebnf
    EnergyOption ::= "--enable-energy-profiling"
                   | "--energy-profile" (* Alias for --enable-energy-profiling *)
                   | "--energy-tdp-profile" "=" ProfileSpec
                   | "--energy-output-format" "=" ("summary" | "json" | "csv")
                   | "--energy-output-file" "=" FilePath
                   | "--energy-loop-iterations" "=" IntegerLiteral
                   | "--energy-branch-bias" "=" ("favor_true" | "favor_false" | "even")
                   | "--energy-verbose"
    ProfileSpec  ::= IDENTIFIER (* e.g., "generic_x86-64" *) 
                   | FilePath   (* Path to a custom TDP profile file *)
    ```

*   **Enabling/Disabling Profiling**:
    *   `--energy-profile` or `--enable-energy-profiling`: A primary flag to enable the energy estimation pass during compilation. Without this, the pass does not run, and no energy overhead is incurred.
    *   Default: Disabled.

*   **Configuration of Profiling Parameters (TBD EP-CONFIG-1)**:
    *   `--energy-tdp-profile=<profile_name_or_path>`: Allows specifying a TDP profile to use (see Section 5.2). `<profile_name_or_path>` could refer to a built-in named profile (e.g., "generic_x86-64", "low_power_arm") or a path to a custom TDP profile file.
        *   Default: A sensible generic profile for the target architecture.
    *   `--energy-uop-weights=<path_to_weights_file>` (Future/Advanced): Potentially allow overriding the default µop weights with a custom file. For v0.1, µop weights will likely be hardcoded or use a fixed internal table.
    *   `--energy-loop-iterations=<N>` (Advanced/Debug): For static analysis, allow overriding the default assumed loop iteration count for unannotated loops. Default: A small, fixed number (e.g., 10).
    *   `--energy-branch-bias=<favor_true|favor_false|even>` (Advanced/Debug): Hint for static branch probability. Default: `even` (50/50).

*   **Output Control**:
    *   `--energy-output-format=<format>`: Specifies the format for detailed energy profiling output if not just integrated into test results.
        *   Options: `summary` (default, e.g., total Joules to stdout), `json`, `csv` (see Section 5.3).
    *   `--energy-output-file=<path>`: Specifies a file to write detailed energy profiling data to, instead of or in addition to stdout.
    *   `--energy-verbose` or `-vE` (conceptual): Enable more verbose output from the energy profiler, possibly including per-function estimates or µop counts per category.

*   **Integration with Test Runner**:
    *   The test runner will use these CLI flags when compiling code for energy-benchmarked tests. Environment variables might also be used to control these settings in CI environments (e.g., `FERRA_ENERGY_PROFILE=true`).

*   **Summary of Energy Profiler CLI Options**: 

    | Flag                                       | Alias / Short                   | Value Type/Example                               | Default                                    | Purpose                                                                 |
    |--------------------------------------------|---------------------------------|--------------------------------------------------|--------------------------------------------|-------------------------------------------------------------------------|
    | `--enable-energy-profiling`                | `--energy-profile`              | N/A (Boolean flag)                               | false                                      | Enables the energy estimation pass.                                     |
    | `--energy-tdp-profile`                     |                                 | `<name>` or `<path>` (e.g., "generic_x86-64") | Target-specific generic profile          | Specifies the TDP profile to use.                                       |
    | `--energy-output-format`                   |                                 | `summary` `json` `csv`                           | `summary`                                  | Sets the output format for detailed profiling data.                     |
    | `--energy-output-file`                     |                                 | `<path>` (e.g., "./energy_report.json")         | (None, output to stdout if format is summary) | Specifies a file for detailed profiling output.                       |
    | `--energy-loop-iterations` (Advanced)      |                                 | `<integer>` (e.g., 10)                           | 10 (Illustrative default, actual TBD EP-UOP-1) | Overrides default assumed loop iterations for static analysis.          |
    | `--energy-branch-bias` (Advanced)          |                                 | `even` `favor_true` `favor_false`              | `even`                                     | Hints static branch probability for analysis.                           |
    | `--energy-verbose`                         | `-vE` (conceptual)              | N/A (Boolean flag)                               | false                                      | Enables more verbose output from the profiler.                          |
    | `--energy-uop-weights` (Future/Advanced)   |                                 | `<path>`                                         | Internal table                             | Overrides default µop weights (not for v0.1).                           |

### 5.2. TDP Profile Management (If applicable)

If the energy model supports different TDP profiles (as discussed in Section 2.3 and configured via `--energy-tdp-profile`), a system for managing these profiles is needed.

*   **Built-in Profiles**:
    *   The compiler could ship with a small set of predefined, generalized TDP profiles for common architectural classes (e.g., "generic-x86-64-desktop", "generic-arm-mobile-a7x", "generic-arm-mobile-x1"). The default profile would be selected based on the compilation target if `--energy-tdp-profile` is not specified. For example, for an x86-64 Linux target, the default might be **`"generic-x86-64-desktop"`**. (Specific default names and their parameters are TBD EP-TDP-1).
    *   These profiles would contain the `TDP_factor` coefficients (e.g., joules per µop_unit, potentially per µop category).
*   **Custom Profile Files**:
    *   Users should be able to provide their own TDP profile files if they have more specific data for their target hardware or want to experiment with different models.
    *   **Format (TBD EP-TDP-1)**: The format for these files needs to be defined (e.g., simple key-value, JSON, TOML). It should specify the TDP_factor(s).
        *   Example (conceptual TOML for custom TDP profile):
            ```toml
            name = "Custom Low-Power ARM"
            architecture_class = "arm-low-power"
            
            [tdp_factors] # Joules per µop_unit
            alu_uop_joules = 0.000000002
            mem_uop_joules = 0.000000010
            fp_uop_joules  = 0.000000005
            # ... other categories or a single factor
            ```
*   **Selection Mechanism**:
    *   The `--energy-tdp-profile` CLI flag would select which profile to use.
    *   If not specified, a default profile based on the compilation target architecture would be chosen.

For v0.1, the number of built-in profiles might be very limited, with the primary focus being on the *mechanism* for estimation using one default profile, and allowing a custom one via a file.

### 5.3. Output Data Format for Profiler

Beyond simple summary output, a structured format for detailed energy profiling data is beneficial for tooling and analysis.

*   **Primary Metric: Total Estimated Joules**: This is the main output required by `Steps.md` for budget checking.
*   **Structured Output (e.g., JSON)**:
    *   When requested via CLI flags (e.g., `--energy-output-format=json --energy-output-file=profile.json`), the profiler could produce a JSON file containing:
        ```json
        {
          "ferra_energy_profile_version": "0.1",
          "timestamp": "2026-01-15T10:30:00Z",
          "compilation_unit": "my_module.ferra", // Or main executable name
          "target_architecture": "x86-64-unknown-linux-gnu",
          "tdp_profile_used": "generic_x86-64-desktop",
          "total_estimated_joules": 65.32,
          "total_weighted_uops": 1234567890, // Optional: total abstract work units
          "uops_by_category": { // Optional: if TDP factors are per category
            "alu": 800000000,
            "memory": 400000000,
            "fp": 34567890
          },
          "functions": [ // Optional: per-function breakdown (Future/Verbose)
            {
              "name": "@my_module::calculate_intensive",
              "estimated_joules": 40.1,
              "weighted_uops": 700000000
            }
            // ... other functions
          ],
          "warnings": [ // Optional: e.g., "Unbounded loop found in @foo, cost may be inaccurate."
            "Static analysis assumed 10 iterations for loop at bar.ferra:L25"
          ]
        }
        ```
*   **CSV Output (Alternative/Simpler)**:
    *   For simpler parsing or spreadsheet import, a CSV format could also be supported, listing key metrics per compilation unit or per function.
*   **Content**:
    *   Essential: Total estimated Joules.
    *   Recommended: TDP profile used, total weighted µops.
    *   Optional/Verbose: µop breakdown by category, per-function estimates, static analysis assumptions made (e.g., loop counts).

This structured output would be consumed by the test runner (for local budget checks) and CI scripts (for CI enforcement and reporting). The exact fields and verbosity levels will be refined based on implementation needs and user feedback.

## 6. Limitations of the Model

It is crucial to understand and acknowledge the inherent limitations of the proposed `µops × TDP_factor` energy estimation model. While providing valuable relative insights, it is not a substitute for direct hardware power measurement and does not capture all aspects of real-world energy consumption.

*   **Estimation, Not Direct Measurement**: The model provides a *static estimation* based on code structure and predefined weights, not a dynamic measurement of actual power consumed during a specific execution on particular hardware.
*   **Factors Not Easily Captured or Modeled (for v0.1)**:
    *   **Dynamic CPU Behavior**: Modern CPUs employ complex dynamic behaviors like Dynamic Voltage and Frequency Scaling (DVFS), turbo boost, and aggressive power gating of idle units, which significantly affect real-time power draw. A static model cannot easily account for these.
    *   **Cache Performance**: The energy cost of memory access is heavily influenced by cache hits and misses. While µop weights for memory operations might include a base cost, predicting cache behavior statically is extremely difficult.
    *   **Memory Subsystem Power**: The model primarily focuses on CPU core energy. The energy consumed by DRAM, memory controllers, and system buses is not directly factored in beyond the µop cost of load/store operations.
    *   **I/O Operations**: Power consumed by disk I/O, network I/O, and other peripherals is outside the scope of this CPU-centric model.
    *   **Operating System Overhead**: Context switches, interrupts, and other OS activities contribute to system energy use but are not modeled by analyzing compiled application code alone.
    *   **Concurrency and Parallelism**: The interaction of multiple threads or processes on multi-core CPUs, including contention for shared resources, can impact energy use in ways not captured by simply summing individual instruction costs.
    *   **Microarchitectural Details**: Fine-grained differences between CPU microarchitectures (even within the same ISA like x86-64) can lead to different real energy costs for the same instruction sequence. The generalized TDP profiles are an abstraction over this.
    *   **Compiler Optimizations**: While the LLVM pass runs late, some very low-level, backend-specific optimizations might still alter the instruction mix or characteristics in ways not perfectly reflected by the µop weights assigned at the LLVM IR level.
*   **Focus on CPU Core Energy**: The initial model predominantly estimates energy related to CPU instruction execution. Total system energy (including GPU, display, network interfaces, etc.) is much broader.
*   **Relative Accuracy Goal**: The primary goal for v0.1 is to provide *relative* accuracy. That is, it should be useful for comparing the energy impact of different code changes within the Ferra language (e.g., "is algorithm A more energy-efficient than algorithm B according to our model?") and for detecting significant regressions against a baseline. Absolute accuracy in Joules against real hardware is a much harder, longer-term goal.

Developers and users of the energy profiler should be aware of these limitations when interpreting its results.

## 7. Future Work

The v0.1 energy profiler design lays a foundation. Several avenues for future work can enhance its accuracy, scope, and usability:

*   **Refinement of µop Weights**:
    *   Conduct empirical studies to derive more accurate µop weights for different LLVM IR instructions on various target architectures.
    *   Develop more sophisticated models for instructions with highly variable costs (e.g., memory access, function calls).
*   **Advanced TDP Models**:
    *   Develop more granular TDP profiles, potentially per CPU model or microarchitecture family.
    *   Incorporate factors beyond a simple scaler, perhaps considering different power states or functional unit utilization.
*   **Calibration with Hardware Power Meters**:
    *   Systematically compare profiler estimates against real-world power measurements from tools like Intel RAPL, PowerTOP, or external hardware power meters for a suite of benchmarks.
    *   Use this data to calibrate and improve the µop weights and TDP models.
*   **Improved Static Analysis for Control Flow**:
    *   Integrate more sophisticated static analysis techniques for estimating loop iteration counts (e.g., based on range analysis) and branch probabilities.
    *   Explore integration with Profile-Guided Optimization (PGO) data (Module 4.2) if/when available to inform these estimates.
*   **Profiling for Other Components**:
    *   **GPU Energy**: Extend estimation to `#[gpu]` annotated code, possibly by modeling SPIR-V operations or interfacing with GPU-specific profiling tools/APIs (Relates to Module 3.5).
    *   **Memory Subsystem**: Develop models for the energy impact of different memory access patterns or DRAM power states.
*   **Support for Different Architectures**:
    *   Extend µop weights and TDP profiles to cover other important target architectures for Ferra (e.g., more ARM variants, RISC-V).
*   **More Granular Reporting and Visualization**:
    *   Provide energy estimates per function, per basic block, or even per source line (requires careful mapping).
    *   Develop tools or IDE integrations to visualize energy "hotspots" in the code.
*   **Integration with Dynamic Analysis (Hybrid Approach)**:
    *   Explore ways to combine static estimates with information from dynamic runtime profiling (e.g., execution counts from instrumented builds) for more accurate overall energy pictures, though this moves beyond the initial "static LLVM pass" model.
*   **Consideration of System-Level Effects**:
    *   Long-term research could investigate models that attempt to incorporate OS effects or interactions with other system components, though this is very challenging.

## 8. Open Questions / TBD

This section consolidates the "To Be Determined" items identified throughout this document, which will require further investigation, prototyping, and decision-making during implementation.

*   **(EP-TDP-1)**: **Source and Granularity of TDP Data**:
    *   What will be the initial set of generalized architecture profiles for TDP factors?
    *   Will the TDP_factor be a single scaler or a set of coefficients per µop category (e.g., ALU, Memory, FP) for v0.1?
    *   What is the defined file format for custom TDP profiles?
*   **(EP-UOP-1)**: **Precise Definition and Counting Methodology for "µops"**:
    *   What is the initial comprehensive table of LLVM IR opcodes and their assigned µop_unit weights for v0.1?
    *   What are the specific heuristics for handling loop iterations (e.g., default count for unannotated loops) and branch probabilities in the static analysis for v0.1?
    *   How will instructions not in the µop weight table be handled (ignored, default weight)?
*   **(EP-TARGET-1)**: **Applicability of the 70J Target from `Steps.md`**:
    *   Which specific reference test case(s) or benchmark suite will the < 70 J CI target initially apply to?
    *   How will this budget be defined and tracked (e.g., as a single test, sum of a suite)?
*   **(EP-CONFIG-1)**: **Extent of Configurability for the Profiler (v0.1)**:
    *   Which of the proposed CLI configuration flags (Section 5.1) will be implemented for v0.1 (e.g., will custom µop weights be supported initially, or just TDP profiles)?
*   **(EP-PASS-IMPL-1)**: **Specific LLVM Pass Implementation Details**:
    *   What is the precise LLVM pass type (e.g., `ModuleAnalysisManager`, `FunctionAnalysisManager`) and its exact insertion point in the pass pipeline?
    *   How will the pass manage and aggregate data across functions and the module?
*   **(EP-OUTPUT-1)**: **Profiler Output Details (v0.1)**:
    *   What specific fields will be included in the structured JSON output for v0.1?
    *   Will per-function estimates be included in the initial verbose output?

Resolving these TBDs will be critical for a successful v0.1 implementation of the Ferra Energy Profiler.

---
This document will guide the design of the Ferra Energy Profiler.
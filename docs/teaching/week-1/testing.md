---
title: "Testing"
duration: "1h"
level: "intermediate"
---

# Testing

> **Duration**: 1 hour
> **Goal**: Learn how to write comprehensive tests for Ferra applications

## Overview

This tutorial covers unit testing, integration testing, property-based testing, and test-driven development in Ferra.

## 1. Unit Testing (15 minutes)

```ferra
// Unit testing implementation
module testing {
    data TestCase {
        name: String
        input: Any
        expected: Any
        setup: Function?
        teardown: Function?
    }

    data TestResult {
        passed: Bool
        actual: Any
        error: String?
        duration: Duration
    }

    fn run_test(test: TestCase) -> TestResult {
        let start_time = now()
        
        // Run setup if provided
        if test.setup {
            test.setup()
        }
        
        let result = match execute_test(test) {
            Ok(actual) => TestResult {
                passed: actual == test.expected,
                actual: actual,
                error: null,
                duration: now() - start_time
            },
            Err(e) => TestResult {
                passed: false,
                actual: null,
                error: e.message,
                duration: now() - start_time
            }
        }
        
        // Run teardown if provided
        if test.teardown {
            test.teardown()
        }
        
        return result
    }

    fn execute_test(test: TestCase) -> Result<Any, TestError> {
        // Execute the test and return result
        return test.input
    }
}
```

## 2. Integration Testing (15 minutes)

```ferra
// Integration testing implementation
module integration {
    data IntegrationTest {
        name: String
        components: List<Component>
        interactions: List<Interaction>
        assertions: List<Assertion>
    }

    data Component {
        name: String
        type: String
        config: Map<String, Any>
    }

    data Interaction {
        from: String
        to: String
        message: Any
        expected_response: Any
    }

    fn run_integration_test(test: IntegrationTest) -> List<TestResult> {
        let results = []
        
        // Setup components
        for component in test.components {
            setup_component(component)
        }
        
        // Run interactions
        for interaction in test.interactions {
            let result = execute_interaction(interaction)
            results.append(result)
        }
        
        // Run assertions
        for assertion in test.assertions {
            let result = verify_assertion(assertion)
            results.append(result)
        }
        
        // Teardown components
        for component in test.components {
            teardown_component(component)
        }
        
        return results
    }

    fn execute_interaction(interaction: Interaction) -> TestResult {
        let component = get_component(interaction.to)
        let response = component.handle(interaction.message)
        
        return TestResult {
            passed: response == interaction.expected_response,
            actual: response,
            error: null,
            duration: now() - start_time
        }
    }
}
```

## 3. Property-Based Testing (15 minutes)

```ferra
// Property-based testing implementation
module property {
    data Property {
        name: String
        generator: Function
        predicate: Function
        iterations: Int
    }

    data PropertyResult {
        passed: Bool
        counterexample: Any?
        iterations: Int
        duration: Duration
    }

    fn test_property(property: Property) -> PropertyResult {
        let start_time = now()
        let iterations = 0
        
        while iterations < property.iterations {
            let input = property.generator()
            if !property.predicate(input) {
                return PropertyResult {
                    passed: false,
                    counterexample: input,
                    iterations: iterations,
                    duration: now() - start_time
                }
            }
            iterations += 1
        }
        
        return PropertyResult {
            passed: true,
            counterexample: null,
            iterations: iterations,
            duration: now() - start_time
        }
    }

    fn generate_random_string() -> String {
        // Generate random string for testing
        return random_string()
    }

    fn generate_random_number() -> Number {
        // Generate random number for testing
        return random_number()
    }
}
```

## 4. Test-Driven Development (15 minutes)

```ferra
// Test-driven development implementation
module tdd {
    data Feature {
        name: String
        description: String
        acceptance_criteria: List<String>
        tests: List<TestCase>
    }

    data DevelopmentCycle {
        feature: Feature
        current_step: DevelopmentStep
        status: DevelopmentStatus
    }

    fn implement_feature(feature: Feature) -> DevelopmentCycle {
        let cycle = DevelopmentCycle {
            feature: feature,
            current_step: DevelopmentStep::WriteTests,
            status: DevelopmentStatus::InProgress
        }
        
        // Write failing tests
        for test in feature.tests {
            write_test(test)
        }
        
        // Implement feature
        cycle.current_step = DevelopmentStep::Implement
        implement_feature_code(feature)
        
        // Refactor
        cycle.current_step = DevelopmentStep::Refactor
        refactor_code()
        
        // Verify all tests pass
        if verify_tests(feature.tests) {
            cycle.status = DevelopmentStatus::Complete
        } else {
            cycle.status = DevelopmentStatus::Failed
        }
        
        return cycle
    }

    fn write_test(test: TestCase) {
        // Write test case
        let test_code = generate_test_code(test)
        write_to_file(test_code)
    }

    fn implement_feature_code(feature: Feature) {
        // Implement feature code
        let code = generate_feature_code(feature)
        write_to_file(code)
    }
}
```

## Quiz

1. What's the first step in test-driven development?
   - A. Write implementation
   - B. Write failing tests
   - C. Refactor code
   - D. Deploy code

2. How should you test component interactions?
   - A. Using unit tests only
   - B. Using integration tests
   - C. Using property tests
   - D. Using manual testing

3. What's the benefit of property-based testing?
   - A. Faster test execution
   - B. Automatic test case generation
   - C. Better test coverage
   - D. Simpler test code

See [DESIGN_DIAGNOSTICS.md](../../reference/DESIGN_DIAGNOSTICS.md) for the quiz schema used in auto-grading.

## Resources

- [Core Semantics](../../reference/CORE_SEMANTICS_V0.1.md)
- [Standard Library](../../reference/STDLIB_CORE_V0.1.md)
- [Coding Standards](../../reference/CODING_STANDARDS.md)
- [Design Diagnostics](../../reference/DESIGN_DIAGNOSTICS.md)

## Next Steps

- [Deployment](./deployment.md)
- [Security](./security.md)
- [Monitoring](./monitoring.md)

## Video Content

- **Duration**: 1 hour
- **Format**: Screen recording with voice-over
- **Sections**:
  1. Introduction (5m)
  2. Unit Testing (15m)
  3. Integration Testing (15m)
  4. Property-Based Testing (15m)
  5. Test-Driven Development (15m)
  6. Conclusion (5m)

## Accessibility Features

- **Subtitles**: `.vtt` file with accurate timestamps
- **Transcript**: Full text transcript in Markdown
- **Code Blocks**: High contrast, syntax highlighted
- **Audio**: Clear, well-paced narration
- **Chapter Markers**: For easy navigation 

#[ai::tag(core_component)]
fn main() {
    let energy_metrics = EnergyMetrics::new();
    let security_context = SecurityContext::new();
} 
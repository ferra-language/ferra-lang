---
number: RFC-XXX
title: "[Short Title]"
status: Draft   # or Review, Accepted, Rejected
version: v0.1
authors: [Name <email>]
last_reviewed: YYYY-MM-DD
last_updated: YYYY-MM-DD
---

# RFC-XXX: [Title]

## Table of Contents
1. [Summary](#1-summary)
2. [Motivation](#2-motivation)
3. [Impact](#3-impact)
   1. [Performance](#31-performance)
   2. [Developer Experience](#32-developer-experience)
   3. [Ecosystem](#33-ecosystem)
4. [Design Decisions](#4-design-decisions)
   1. [Decision 1](#41-decision-1)
   2. [Decision 2](#42-decision-2)
5. [Drawbacks](#5-drawbacks)
   1. [Category 1](#51-category-1)
   2. [Category 2](#52-category-2)
6. [Security & Privacy](#6-security--privacy)
   1. [Threat Model](#61-threat-model)
   2. [Permissions Impact](#62-permissions-impact)
   3. [Audit Requirements](#63-audit-requirements)
   4. [Data Handling](#64-data-handling)
   5. [Error Reporting](#65-error-reporting)
7. [Implementation Plan](#7-implementation-plan)
   1. [Phase 1: Name](#71-phase-1-name)
   2. [Phase 2: Name](#72-phase-2-name)
   3. [Issue Mapping](#73-issue-mapping)
8. [Migration Strategy](#8-migration-strategy)
9. [Unresolved Questions](#9-unresolved-questions)
10. [Future Possibilities](#10-future-possibilities)
11. [References](#11-references)

## 1. Summary
[One paragraph explaining the proposal in simple terms]

## 2. Motivation
[Why we need this feature/change]
- Problem statement
- Current limitations
- User impact

## 3. Impact
### 3.1 Performance
- Compilation time impact
  - **Takeaway**: [Key performance trade-off]
- Runtime performance
  - **Takeaway**: [Key runtime trade-off]
- Memory usage
  - **Takeaway**: [Key memory trade-off]
- Binary size
  - **Takeaway**: [Key size trade-off]

### 3.2 Developer Experience
- Learning curve
  - **Takeaway**: [Key learning curve insight]
- Code readability
  - **Takeaway**: [Key readability insight]
- Tooling support
  - **Takeaway**: [Key tooling insight]

### 3.3 Ecosystem
- Library compatibility
- Tool integration
- Community adoption

## 4. Design Decisions
### 4.1 [Decision 1]
- **Proposed Solution**: [Brief description]
- **Rationale**: [Why this approach]
- **Alternatives Considered**:
  1. [Alternative 1]
     - Pros:
     - Cons:
     - Why rejected:
  2. [Alternative 2]
     - Pros:
     - Cons:
     - Why rejected:

### 4.2 [Decision 2]
[Same structure as above]

## 5. Drawbacks
### 5.1 [Category 1]
- [Explicit drawback 1]
- [Explicit drawback 2]

### 5.2 [Category 2]
- [Explicit drawback 3]
- [Explicit drawback 4]

## 6. Security & Privacy
### 6.1 Threat Model
- **Threat Model**: [Description of security considerations]
  - [Threat 1]: [Mitigation strategy]
  - [Threat 2]: [Mitigation strategy]

### 6.2 Permissions Impact
- **Permissions Impact**: [How this affects system permissions]

### 6.3 Audit Requirements
- **Audit Requirements**: [What needs to be audited]

### 6.4 Data Handling
- **Data Handling**: [How user data is affected]

### 6.5 Error Reporting
- **Error Reporting**: [Link to error reporting schema]

## 7. Implementation Plan
### 7.1 Phase 1: [Name]
- **Timeline**: [Target quarter/year]
- **Deliverables**:
  - [Deliverable 1]
  - [Deliverable 2]
- **Dependencies**: [List of dependencies]

### 7.2 Phase 2: [Name]
[Same structure as above]

### 7.3 Issue Mapping
| Phase   | Issue           | URL                                    | Priority |
| ------- | --------------- | -------------------------------------- | -------- |
| Phase 1 | #XXX (TAG-1)    | https://github.com/org/repo/issues/XXX | High     |
| Phase 2 | #XXX (TAG-2)    | https://github.com/org/repo/issues/XXX | Medium   |

## 8. Migration Strategy
- **Backward Compatibility**: [Impact on existing code]
- **Breaking Changes**: [List of breaking changes]
- **Migration Tools**: [Available tools]
- **Documentation Updates**: [Required docs]
- **Runtime Impact**: [Runtime requirements]
- **Migration Path**:
  - [Step 1]
  - [Step 2]
  - **Takeaway**: [Key migration insight]

## 9. Unresolved Questions
1. [Question 1] (TAG-1) - [Priority]
   - Current thinking: [Current position]
   - Open issues: [List of open issues]
   - Next steps: [Action items]

2. [Question 2] (TAG-2) - [Priority]
   [Same structure as above]

## 10. Future Possibilities
1. [Future enhancement 1]
2. [Future enhancement 2]

## Example Section
Here's an example of how to fill out a section:

### Actor Model
- **Proposed Solution**: Deterministic actor system with compile-time scheduling
- **Rationale**: 
  - Prevents data races by design
  - Enables reproducible behavior
  - Simplifies reasoning about concurrent code
- **Alternatives Considered**:
  1. Traditional Threading
     - Pros: Familiar, direct OS integration
     - Cons: Complex synchronization, non-deterministic
     - Why rejected: Too error-prone, hard to reason about
  2. Pure Async/Await
     - Pros: Simple, familiar to many developers
     - Cons: Limited scalability, complex error handling
     - Why rejected: Insufficient for complex systems

## 11. References
1. [Reference 1]
2. [Reference 2]
3. [Link to relevant section in Steps.md]
4. [Link to related design docs] 
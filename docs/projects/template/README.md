# Project Templates

Templates for systematic CardioTrust improvements using the three-phase AI-assisted development workflow.

## Usage

1. **Copy templates** to new project directory:
   ```
   docs/projects/improvement-name/
   ├── spec-template.md → improvement-spec.md
   └── plan-template.md → plan.md
   ```

2. **Replace placeholders**:
   - `[IMPROVEMENT_NAME]` → Your improvement name
   - `[improvement description]` → Specific descriptions  
   - `[X-Y] hours` → Time estimates

## Template Structure

**`spec-template.md`** - Research software improvement specification:
- Problem analysis and technical debt identification
- CardioTrust-specific scope (algorithms, GPU, UI, config, etc.)
- Algorithm safety and research integrity requirements
- Performance preservation criteria
- Risk assessment framework

**`plan-template.md`** - Phase-based implementation plan:
- Task breakdown with algorithm safety protocols
- GPU/CPU implementation coordination
- Performance monitoring checkpoints
- Research reproducibility validation

## Critical Requirements

**Algorithm Safety**:
- Preserve mathematical correctness
- Maintain GPU/CPU consistency  
- Ensure research reproducibility
- Performance impact ±5% baseline

**Risk Levels**:
- **High**: Core algorithm logic, mathematical operations
- **Medium**: Data structures, configuration systems  
- **Low**: UI, documentation, non-algorithm code

**Validation**:
- Tests pass (two-run requirement)
- Benchmarks within acceptable bounds
- Scenarios execute reproducibly
- GPU/CPU results consistent

## Common Improvement Types

- **Code Quality**: Error handling, organization, documentation
- **Performance**: Algorithm optimization, memory usage
- **GPU**: Kernel improvements, CPU/GPU consistency
- **UI/UX**: Native/WASM interface polish
- **Testing**: Coverage, benchmarks
- **Architecture**: Module refactoring, cleanup
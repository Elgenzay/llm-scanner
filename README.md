# LLM Scanner

An LLM jailbreak scanner with configurable detection methods.

Requires [VHACK](https://github.com/ZioSec/VHACK) running at `http://localhost:8000` (if `mock_mode = false`)

## Quick Start
```bash
# Clone and build
git clone git@github.com:Elgenzay/llm-scanner.git
cd llm-scanner

# Run with default settings (mock mode enabled)
cargo run
```

## Configuration

On first run, the scanner automatically generates:
- `config.toml` - Main configuration file
- `data/` - Directory containing all data files

All configuration options can be overridden via command line:
```bash
cargo run -- \
  --target http://localhost:8000/api/chat \
  --prompts data/prompts.csv \
  --concurrency 4 \
  --timeout-ms 30000 \
  --out report.jsonl \
  --detection-method llm \
  --mock-mode false
```

### Data Files (`data/`)

All files are auto-generated with [defaults](https://github.com/Elgenzay/llm-scanner/tree/main/defaults) on first run and can be customized:

- **prompts.csv** - List of prompts to test (one per line)
- **safe_patterns.csv** - Patterns indicating safe/refused responses
- **unsafe_patterns.csv** - Patterns indicating jailbreak success
- **test_responses.csv** - Mock responses for testing (used when `mock_mode = true`)
- **judge_prompt.md** - Prompt template for LLM-based evaluation

**Mock Mode Behavior:** Responses in `data/test_responses.csv` correspond by line number to prompts in `data/prompts.csv`. The first prompt gets the first response, etc.

## Detection Methods

### Pattern
Fast keyword matching using safe and unsafe pattern files. Defaults to safe if no patterns match.

### LLM
Uses the target LLM itself as a judge to evaluate if responses contain jailbreaks. Reads evaluation criteria from `data/judge_prompt.md`.

### Hybrid (Default)
Attempts pattern matching first. If no pattern matches (neither safe nor unsafe), falls back to LLM evaluation.

## Output

The scanner generates two output files to `output/`:

- **report.jsonl** - Results in JSONL format
- **summary.html** - HTML summary

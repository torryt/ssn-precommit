# ssn-precommit

A [pre-commit](https://pre-commit.com/) hook that scans staged changes for potential 11-digit SSNs before committing.

It only checks **added lines** in the diff — not entire files.

Supported formats:
- `01234567890` (11 consecutive digits)
- `012345 67890` (6 digits, space, 5 digits)

When SSNs are found, it blocks the commit and tells you how to bypass.

## Installation

Add this to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/torryt/ssn-precommit
    rev: v0.4.0
    hooks:
      - id: ssn-check
```

Then run:

```sh
pre-commit install
```

That's it. The hook will now run on every `git commit` across any repo where you add this config.

## What it does

1. Receives the list of staged files from pre-commit
2. Runs `git diff --cached` on each file to get only changed lines
3. Scans added lines for 11-digit SSN patterns (with or without a space after the 6th digit)
4. If matches are found, blocks the commit and displays the matches

## Options

Pass options via `args` in your `.pre-commit-config.yaml`:

```yaml
hooks:
  - id: ssn-check
    args: ['--no-mask', '--ignore', '00123456789,12345678901']
```

| Flag | Description |
|------|-------------|
| `--no-mask` | Show full SSNs in output (default: masked, e.g. `001234*****`) |
| `--ignore` | Comma-separated list of numbers to ignore |

## Ignore file

You can also create a `.ssnignore` file in the repo root with one number per line:

```
# Test customer IDs - these are not real SSNs and safe to commit
00123456789
12345678901
```

Lines starting with `#` are treated as comments.

## Bypassing

```sh
git commit --no-verify
```

## Requirements

- [pre-commit](https://pre-commit.com/#install) (`pip install pre-commit`)
- Rust toolchain (pre-commit will build the binary automatically via `language: rust`)

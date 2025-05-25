# Pull Request

Thank you for your contribution! Please ensure your PR meets the following:

## Description

- [ ] Is this a bug fix, feature, or docs change?
- [ ] Link to any related issues (e.g. “Fixes #123”).

## Type of change

_Select one or more:_

- `feat:` A new feature
- `fix:` A bug fix
- `docs:` Documentation only changes
- `style:` Formatting, white-space, etc.
- `refactor:` Code change that neither fixes a bug nor adds a feature
- `test:` Adding missing tests or correcting existing tests
- `chore:` Build process or auxiliary tool changes

**Before pushing, please run:**
```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
```

## Checklist

- [ ] I've updated `CHANGELOG.md` (if applicable).
- [ ] I've added tests for my changes.
- [ ] All new and existing tests pass.
- [ ] I've run `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings`.

## Additional Notes

Anything else to note or discuss?
 
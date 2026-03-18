# Project Constraints

This directory contains immutable constraints that must not be violated.

## File Naming Convention

```
<category>-<name>.md
```

Categories: security, api, code-style, architecture, other

## Template

```markdown
# Constraint: Name

- **Category**: security | api | code-style | architecture | other
- **Severity**: critical | high | medium
- **Created**: YYYY-MM-DD

## Description

Clear description of the constraint.

## Scope

- Applies to: `glob pattern`
- Excludes: `glob pattern`

## Rationale

Why does this constraint exist?

## Exceptions

When can this be bypassed?
```

## Required Fields

- Category, Severity, Description, Scope, Rationale

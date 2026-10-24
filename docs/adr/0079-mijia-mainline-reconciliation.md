# ADR-0079: Reconcile Mijia and mainline native contracts

Status: accepted for implementation.
Date: October 24, 2026.

## Context

The owner requested a linear update of the Mijia worktree from current mainline.
Independent development used API 1.24/1.25 for different features and duplicated
ADR numbers. Numeric version checks could therefore misidentify Mijia support.

## Decision

The combined contract is API 1.27.0. It preserves mainline's identity-recovery
functions, exact Meiju profiles, dishwasher grants and all Mijia account/fan
features. Mijia-aware Flutter startup and provider/grant negotiation require 1.27.
Meiju identity restoration remains Meiju-only in combined sessions. Explicit
foreign-provider recovery requests fail before login or credential mutation.

The three branch-only Mijia decisions are renumbered ADR-0076 through ADR-0078.
Their decisions remain unchanged. Earlier phase reports retain historical API
numbers and test evidence; these are not feature negotiation for current binaries.
No merge commit or remote history rewrite is required for the local rebase.

## Compatibility

There are still 34 ordinary methods and 19 C functions plus the Android initializer.
SQLite schema 5, Mijia payload 1 and Meiju payload 3 are unchanged. Rebuild native
and Flutter packages together. No account, appliance or physical-phone acceptance
is established by the combined fictional regressions.

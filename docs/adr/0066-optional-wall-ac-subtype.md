# ADR-0066: Resolve reviewed wall-AC temperature without an optional subtype

Status: accepted for the owner-requested temperature integration correction.

## Problem and evidence

The selected wall AC has protected product-code and wall-category evidence, but
its saved inventory lacks a nonzero subtype. A temporary Android diagnostic
reported only those presence booleans; no product values or identifiers were
exported. ADR-0041 currently refuses before considering any lower-priority mapping.
This leaves a readable target with no setter despite the remaining evidence.

The pinned upstream [mapping resolver](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/__init__.py)
checks subtype only when present, then product code, category and family default.
The [AC mappings](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/device_mapping/T0xAC.py)
identify the subtype-524 portable override with a shared placeholder product code.
HomeVerb need not require that optional field for every reviewed wall model, but
must not introduce a family-wide fallback or infer a subtype value.

## Narrow correction

When the protected subtype is absent, permit the existing product/category
temperature selection only if all of these conditions hold:

- The independently saved exact model is one of the four existing reviewed AC
  models; descriptor model text alone never establishes this condition.
- A protected non-placeholder product code is present. The shared `00000Q1F`
  placeholder does not establish a wall mapping and remains refused.
- The protected category explicitly normalizes to `wall_air_conditioner`.
- The existing product-specific selection accepts the mapping. All excluded
  centralized/water/fan profiles remain excluded before category fallback.

Known subtypes keep their original priority, including the portable override.
Missing model, unknown model, missing category/product and ambiguous placeholders
remain denied. Do not insert a synthetic subtype into stored evidence. Apply the
same resolver to saved control options, admitted descriptors, protected execution
checks and read decoding. Keep exact grants, arguments, bounds, steps and one- or
two-field encoding. No power/mode field, cached status, replay or fallback is added.

## Compatibility and verification

Native API 1.22.0, its schemas, SQLite schema 5 and protected payload 3 are
unchanged. The existing climate action schema already represents the selected
bounds; this correction only makes a previously omitted action available for the
narrow evidence combination. Ordinary requests still cannot enable controls.
Existing `CAPABILITY_UNSUPPORTED`, `accepted` and `unknown` outcomes retain their
meaning. Physical acceptance must be recorded separately from the synthetic suite.

Test product overrides, category normalization, missing/unknown identity,
placeholders and excluded profiles, exact encoding, option/descriptor consistency,
independent grants and weakened public descriptors. Remove all temporary diagnostic
logging before committing or delivering the corrected APK.

# Flutter UI paused checkpoint

Date: October 16, 2026. Owner decision: pause the current goal and pixel-equality
work, preserve the branch and worktree, and record a checkpoint for later resumption.
This is not a completed visual acceptance or a waiver of the original target.

## Locations and baseline

- Worktree: `/home/arren/.codex/worktrees/homeverb-flutter/homeverb`.
- Retained branch: `codex/flutter-ui`.
- Remote: `https://github.com/arrenxxxxx/homeverb.git`.
- Last verified application commit: `d3d3395aea6c789449a77aa6f207098b10f1e92f`.
  It was pushed and the remote tip was read back successfully.
- Prior native form-alignment commit: `6b588565c52737378fc19418aad92b143c5e2b81`.
- Pinned Core: `7db8e9bc3e0c2be8ced4d2ab5ffb963d35df8987`; native API 1.14.1.
- Flutter: FVM 4.3.1, Flutter 3.47.7. Run `/home/arren/.local/bin/fvm` from
  `apps/mobile`, where `.fvmrc` is present.
- Full executed-check history: [verification.md](verification.md).
- Native contract: [native-ui-contract.md](../../docs/native-ui-contract.md).

The main checkout has independent Core work. Do not copy or reset its changes.
Preserve the frontend's unrelated untracked `.idea/` directory. Retain the branch;
there is no authorization to merge it, remove it or delete this worktree.

## Verified application checkpoint

At `d3d3395`, all 84 Flutter tests and static analysis passed. The suite includes
82 native-widget captures, 201 explicitly simulated prototype branches, native
Mock/fixture interactions, inputs, high contrast and enlarged text. Simulated
branches are not real provider operations.

The ordinary Android dual-ABI development APK was built, installed and exercised
on Android 15 API 35 x86_64. Actual pointer checks covered navigation, Search,
Back, Add Device, explicit Mock discovery, card opening, state refresh, mode
selection and cancellation. The earlier input phase also verified OS keyboard
input, native text selection, save/cancel and cold-start persistence.

The APK is `output/flutter-verification/android/homeverb-0.2.0-android-development-release.apk`.
SHA-256: `3ef0c9f3598e7025cea550a8f1e3b70c8df96b9551d8dd6539338dbbbae2ca2b`.
Its source and evidence are recorded in `android/release-artifact.json`,
`android/glyph-package-check.json` and `android/glyph-release-clicks.json` under
`output/flutter-verification/`. Both Core ABIs remained unchanged, packaged assets
matched source, and 16 KiB ZIP alignment passed. This APK excludes the uncommitted
surface experiment described below.

At that commit the exact screenshot gate remained **0/82**, with mean channel
error **0.6451/255**. All 82 states improved over the preceding phase. Four cropped
D01 icon regions matched exactly; this does not establish whole-screen equality.
No real account, appliance, physical phone, iOS or new Keystore acceptance was
performed. Full Android screen-reader acceptance also remains unverified.

## Preserved, uncommitted surface experiment

The worktree intentionally retains these unfinished changes:

- `apps/mobile/lib/skin.dart`: decorative corner masks, inclusive border painting,
  rounded physical extents and the existing gradient, with native child widgets
  and high-contrast fallback preserved. Shadow rendering is unchanged.
- `apps/mobile/tool/generate_surface_assets.py`.
- 24 `apps/mobile/assets/surface-*.png` corner and border masks.

Only the 82-state capture test and static analysis have passed for this experiment.
The full suite, Android build, actual GPU rendering and native clicks have not
been rerun. Do not label the experiment verified or use the existing APK as its
runtime evidence.

Current experimental comparison: **0/82 exact**, mean channel error **0.5268/255**;
73 states improved, 8 were unchanged and S11 regressed by 0.0039/255. The live
`output/flutter-verification/index.html` currently displays this experiment,
not the last fully verified commit. Reference images and the strict gate were
not changed.

Known next investigation: the prototype rounds each absolute outline endpoint
independently. The experiment currently rounds the origin and extent separately.
For a fractional origin and height these can disagree by one pixel. Correct that
before acceptance, then investigate the reduced S11 skin preview. Do not change
references or tolerance to hide differences. No further implementation was done
after the owner requested this pause.

## Local archive

`output/flutter-verification/checkpoints/flutter-ui-2026-10-16.tar.gz` preserves:

- The verified source commit ID, a binary-capable Git patch and all 25 new files.
- A full copy of the modified Dart file for inspection.
- The current comparison report, reference/actual/difference images and metrics.
- The previous verified capture metrics and screenshots.
- The verified APK, provenance and relevant actual Android click evidence.
- Surface experiment logs and a copy of this handoff.

The adjacent `.sha256` file records the archive checksum. `MANIFEST.json` inside
the archive records each member's SHA-256. The archive remains local and is not
uploaded. The existing worktree already contains the experiment; do not apply
the patch on top of it.

To restore elsewhere, first create a clean checkout at the recorded application
commit, extract the archive outside that checkout, check `git apply --check`
against `working-tree.patch`, apply it once, and copy `new-files/` into the
checkout preserving relative paths. Do not restore over unrelated changes.
Build products and SDKs are not a substitute for rebuilding source.

## Resume checklist

1. Resume only when the owner asks. Re-read this note, `verification.md`, the
   current worktree status and native contract; do not assume main's Core is the
   same pinned version. Keep unavailable operations explicit as agreed.
2. Preserve or restore the surface experiment; fix endpoint rounding and S11
   scaling before accepting it. Inspect the actual diffs and screenshots.
3. Run FVM analysis and the appropriate Flutter tests. Then generate the report
   from the repository root with
   `python3 apps/mobile/tool/compare_screens.py --require-exact`.
   A nonzero exit is expected while any screen differs.
4. Verify any changed GPU rendering in an ordinary Android APK. SDK location is
   `/tmp/homeverb-flutter-sdk`; JDK is
   `/home/arren/.sdkman/candidates/java/21.0.11-tem`; the isolated AVD is
   `/tmp/homeverb-flutter-avd/homeverb-api35.avd`. Recheck these temporary paths
   and runtime state before use; no running process is guaranteed at resumption.
5. Continue the remaining surface, typography, primitive and layout differences.
   P08 busy/empty/partial/failed/interrupted variants, D03's extra clear-search
   control, and complete functionality/device acceptance remain open.
6. Commit each verified phase separately, push the retained frontend branch as
   already authorized, and read back the remote tip. Do not claim the original
   goal complete until its full acceptance criteria are evidenced.

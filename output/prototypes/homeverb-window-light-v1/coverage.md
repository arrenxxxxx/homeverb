# UI coverage map

Scope: the owner-approved mobile requirements and room/card/platform/sharing refinements. These locally rendered Window Light panels are visual design evidence, not implementation or hardware evidence.

| Requirement | Rendered screens |
|---|---|
| No HomeVerb login; direct first use | [D04](screens/D04.png) |
| Exactly three primary destinations | [D01](screens/D01.png) · [P01](screens/P01.png) · [S01](screens/S01.png) |
| Consumer-facing platform naming | [P01](screens/P01.png) · [P02](screens/P02.png) · [P09](screens/P09.png) |
| Room-filtered two-column cards | [D01](screens/D01.png) · [D02](screens/D02.png) |
| Custom room creation and validation | [R01](screens/R01.png) · [R02](screens/R02.png) · [R05](screens/R05.png) |
| Room renaming, ordering and deletion | [R01](screens/R01.png) · [R04](screens/R04.png) · [R05](screens/R05.png) |
| Move devices without changing authority | [R03](screens/R03.png) · [D06](screens/D06.png) |
| Unassigned and empty-room handling | [R01](screens/R01.png) · [R03](screens/R03.png) · [D09](screens/D09.png) |
| Search and card quick actions | [D03](screens/D03.png) · [D05](screens/D05.png) |
| Favorites and card order | [D05](screens/D05.png) · [D10](screens/D10.png) |
| Local device name and connection preference | [D06](screens/D06.png) · [D07](screens/D07.png) · [D08](screens/D08.png) |
| Capability-driven climate controls | [C01](screens/C01.png) · [C02](screens/C02.png) · [C14](screens/C14.png) |
| Lighting, fan and media controls | [C05](screens/C05.png) · [C06](screens/C06.png) · [C07](screens/C07.png) |
| Components and read-only measurements | [C08](screens/C08.png) · [C13](screens/C13.png) |
| Consequential action confirmation | [C09](screens/C09.png) · [C10](screens/C10.png) · [C11](screens/C11.png) |
| Accepted, verified, failed, unknown and partial | [C03](screens/C03.png) · [C04](screens/C04.png) · [C11](screens/C11.png) · [C12](screens/C12.png) · [S05](screens/S05.png) · [S06](screens/S06.png) |
| Requested values do not replace observations | [C03](screens/C03.png) · [C04](screens/C04.png) · [C14](screens/C14.png) |
| Cached, missing and stale states | [D11](screens/D11.png) · [C13](screens/C13.png) |
| Device discovery and permission request | [A01](screens/A01.png) · [A02](screens/A02.png) · [A03](screens/A03.png) · [A04](screens/A04.png) |
| Ambiguous identity requires a decision | [A05](screens/A05.png) |
| Device selection and room assignment | [A04](screens/A04.png) · [A06](screens/A06.png) · [R03](screens/R03.png) |
| Discovery timeout and denied permissions | [A07](screens/A07.png) · [A08](screens/A08.png) |
| Multiple platform accounts and regions | [P01](screens/P01.png) · [P03](screens/P03.png) · [P09](screens/P09.png) |
| Provider browser/code/device authorization | [P04](screens/P04.png) · [P05](screens/P05.png) · [P06](screens/P06.png) |
| Authorization expiry and reconnection | [P07](screens/P07.png) · [P08](screens/P08.png) · [P10](screens/P10.png) |
| Local removal distinct from remote revocation | [P11](screens/P11.png) · [P12](screens/P12.png) |
| Sharing only through Settings | [S01](screens/S01.png) · [H01](screens/H01.png) · [H05](screens/H05.png) |
| Explicit nearby receiving and peer selection | [H02](screens/H02.png) · [H05](screens/H05.png) |
| Pair-code comparison and mismatch rejection | [H03](screens/H03.png) · [H06](screens/H06.png) · [H14](screens/H14.png) |
| Actual share scope, expiry and revocation | [H01](screens/H01.png) · [H04](screens/H04.png) |
| Bilateral consent and account conflicts | [H06](screens/H06.png) · [H07](screens/H07.png) |
| Separate transfer, import and validation | [H08](screens/H08.png) · [H09](screens/H09.png) · [H12](screens/H12.png) · [H16](screens/H16.png) |
| No plaintext fallback; ineligible grants | [H10](screens/H10.png) · [H11](screens/H11.png) |
| Timeout, interruption and no blind resend | [H12](screens/H12.png) · [H13](screens/H13.png) · [H15](screens/H15.png) |
| Independent recipient operation | [H08](screens/H08.png) |
| One Window Light skin, local name and connection defaults | [S02](screens/S02.png) · [S03](screens/S03.png) · [S10](screens/S10.png) · [S11](screens/S11.png) |
| Local data controls and scoped deletion | [S04](screens/S04.png) · [S09](screens/S09.png) |
| Recent outcomes and timestamped evidence | [S05](screens/S05.png) · [S06](screens/S06.png) |
| Redacted diagnostic preview before export | [S07](screens/S07.png) |
| About and honest compatibility | [S08](screens/S08.png) · [S12](screens/S12.png) · [P13](screens/P13.png) |
| Large-text and wide layout adaptation | [D12](screens/D12.png) · [W01](screens/W01.png) · [W02](screens/W02.png) |

## Window Light acceptance additions

| Requirement | Evidence / remaining acceptance |
|---|---|
| Apply one visual system to every state | All 82 panels use the shared locally rendered background, surface and accent tokens. |
| Exactly one skin option | S02 and S11 use stable ID `window_light`; no alternate/dark/system option. |
| Preserve future skin extensibility | [Skin contract](../../../docs/ui-window-light.md) and [Flutter app](../../../apps/mobile/README.md); local preference persistence is implemented, while additional skins remain out of scope. |
| Keep development coverage explicit | [Functional handoff](../../../docs/ui-functional-spec.md), [82-state inventory](../../../docs/ui-screen-inventory.md) and 15 acceptance groups. |
| Honest rendering evidence | Local Python composition; no Flutter build, browser UI test or physical-device verification is implied. |

## Complete screen inventory

| Board | Screens |
|---|---|
| [Everyday devices](boards/01-devices.png) | [D01: All devices](screens/D01.png) · [D02: Room filter](screens/D02.png) · [D03: Find a device](screens/D03.png) · [D04: First launch](screens/D04.png) |
| [Make room for your home](boards/02-rooms.png) | [R01: Room management](screens/R01.png) · [R02: Create a room](screens/R02.png) · [R03: Move devices](screens/R03.png) · [R04: Delete a room](screens/R04.png) |
| [A place for every device](boards/03-device-details.png) | [D05: Card actions](screens/D05.png) · [D06: Device information](screens/D06.png) · [D07: Rename a device](screens/D07.png) · [D08: Connection choice](screens/D08.png) |
| [Comfort, clearly controlled](boards/04-climate.png) | [C01: Climate control](screens/C01.png) · [C02: Choose a mode](screens/C02.png) · [C03: Accepted, awaiting state](screens/C03.png) · [C04: Unknown outcome](screens/C04.png) |
| [Controls that fit the appliance](boards/05-capability-controls.png) | [C05: Lighting](screens/C05.png) · [C06: Fan](screens/C06.png) · [C07: Media](screens/C07.png) · [C08: Components and sensors](screens/C08.png) |
| [Know what happened](boards/06-actions-results.png) | [C09: Appliance cycle](screens/C09.png) · [C10: Confirm a cycle](screens/C10.png) · [C11: Verified state](screens/C11.png) · [C12: Partial result](screens/C12.png) |
| [Bring appliances together](boards/07-add-devices.png) | [A01: Choose an add method](screens/A01.png) · [A02: Local permission](screens/A02.png) · [A03: Discovering](screens/A03.png) · [A04: Select discovered devices](screens/A04.png) |
| [Finish adding, or recover](boards/08-discovery-recovery.png) | [A05: Resolve a possible duplicate](screens/A05.png) · [A06: Devices added](screens/A06.png) · [A07: Nothing found](screens/A07.png) · [A08: Permission denied](screens/A08.png) |
| [Connect your appliance platforms](boards/09-platforms.png) | [P01: Platform accounts](screens/P01.png) · [P02: Select a platform](screens/P02.png) · [P03: Choose region and account](screens/P03.png) · [P04: Native browser consent](screens/P04.png) |
| [Authorization, step by step](boards/10-authentication.png) | [P05: Verification code](screens/P05.png) · [P06: Approve on the appliance](screens/P06.png) · [P07: Authorization expired](screens/P07.png) · [P08: Account retrieval result](screens/P08.png) |
| [Manage access deliberately](boards/11-platform-details.png) | [P09: Platform details](screens/P09.png) · [P10: Reconnect needed](screens/P10.png) · [P11: Remove local account](screens/P11.png) · [P12: Revoke provider access](screens/P12.png) |
| [Quiet, practical settings](boards/12-settings.png) | [S01: Settings home](screens/S01.png) · [S02: Appearance and skin](screens/S02.png) · [S03: Connection preferences](screens/S03.png) · [S04: Local data](screens/S04.png) |
| [Send access nearby](boards/13-send-authorization.png) | [H01: Choose authorization](screens/H01.png) · [H02: Find a recipient](screens/H02.png) · [H03: Verify the recipient](screens/H03.png) · [H04: Review before sending](screens/H04.png) |
| [Receive and connect independently](boards/14-receive-authorization.png) | [H05: Ready to receive](screens/H05.png) · [H06: Review incoming access](screens/H06.png) · [H07: Existing account conflict](screens/H07.png) · [H08: Import result](screens/H08.png) |
| [Make sharing outcomes explicit](boards/15-transfer-states.png) | [H09: Transferring](screens/H09.png) · [H10: Secure storage unavailable](screens/H10.png) · [H11: Cannot share this account](screens/H11.png) · [H12: Receipt not confirmed](screens/H12.png) |
| [Understand and recover](boards/16-activity-support.png) | [S05: Recent operations](screens/S05.png) · [S06: Operation evidence](screens/S06.png) · [S07: Diagnostic preview](screens/S07.png) · [S08: About and compatibility](screens/S08.png) |
| [Recover without repeating secrets](boards/17-sharing-recovery.png) | [H13: No recipient found](screens/H13.png) · [H14: Codes do not match](screens/H14.png) · [H15: Session stopped](screens/H15.png) · [H16: Imported, needs authorization](screens/H16.png) |
| [Small details that keep it simple](boards/18-edge-states.png) | [R05: Room name validation](screens/R05.png) · [D09: An empty room](screens/D09.png) · [P13: Platform unavailable](screens/P13.png) · [S09: Clear local data confirmation](screens/S09.png) |
| [Make it feel like yours](boards/19-preferences-order.png) | [S10: This device](screens/S10.png) · [S11: Window Light skin preview](screens/S11.png) · [S12: Compatibility](screens/S12.png) · [D10: Reorder device cards](screens/D10.png) |
| [The same home, with more space](boards/20-adaptive-layouts.png) | [W01: Tablet device home](screens/W01.png) · [W02: Tablet device controls](screens/W02.png) |
| [Clear even when information is missing](boards/21-state-accessibility.png) | [D11: Cached device state](screens/D11.png) · [C13: Unknown sensor reading](screens/C13.png) · [C14: Invalid control value](screens/C14.png) · [D12: Larger text layout](screens/D12.png) |

# Window Light screen inventory

Status: visual specification, October 15, 2026. All 82 states below are rendered in the [current atlas](../output/prototypes/homeverb-window-light-v1/index.html). Actions navigate review images; the atlas does not run appliance commands or authenticate accounts. See [functional requirements and acceptance](ui-functional-spec.md) and [Flutter/skin implementation](ui-window-light.md).

Each entry preserves its stable screen ID, visible content and explicit branch targets. Static explanatory copy is included to make safety and recovery requirements reviewable. System dialogs, keyboard variants and generic loading/save-error variants reuse these components; they remain required acceptance cases even when no separate image ID exists. Unlisted Back/Cancel behavior returns to the actual caller, as defined by the functional handoff.

The legacy S11 dark-appearance study is superseded by the one-option Window Light preview. There is no alternate skin in this release.

The October 15 Meiju account-flow amendment below changes P08 to automatic cloud device retrieval after authorization. The existing P08 image and atlas branch buttons still show the earlier manual discovery flow. The required transitions are recorded below for implementation; visual assets and interactive atlas updates remain pending. Other screen IDs retain their meaning.

## Everyday devices

### D01 — All devices

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D01.png) · Root: Devices

HomeVerb header with search and plus icons. Room chips All (selected), Living Room, Bedroom, Bathroom; small manage icon. Two-column four-card grid: Bedroom AC / Cooling / 24 C / On; Floor lamp / 65% / On; Living room TV / Off; Air sensor / 22 C / 46%. Small freshness line Updated just now; subtle Refresh action. Separate terracotta quick power targets on writable cards, sensor has none. Bottom nav Devices (selected), Platforms, Settings. No profile avatar.

Visible content: Room filters; device cards; observed state/freshness; explicit refresh; separate quick actions; three-root navigation.

| Action or outcome | Review destination |
|---|---|
| Select a room | [D02](../output/prototypes/homeverb-window-light-v1/screens/D02.png) |
| Search | [D03](../output/prototypes/homeverb-window-light-v1/screens/D03.png) |
| Add device | [A01](../output/prototypes/homeverb-window-light-v1/screens/A01.png) |
| Manage rooms | [R01](../output/prototypes/homeverb-window-light-v1/screens/R01.png) |
| Open AC | [C01](../output/prototypes/homeverb-window-light-v1/screens/C01.png) |
| Open lamp | [C05](../output/prototypes/homeverb-window-light-v1/screens/C05.png) |
| Card menu | [D05](../output/prototypes/homeverb-window-light-v1/screens/D05.png) |
| Platforms | [P01](../output/prototypes/homeverb-window-light-v1/screens/P01.png) |
| Settings | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |
| Inspect cached state | [D11](../output/prototypes/homeverb-window-light-v1/screens/D11.png) |

### D02 — Room filter

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D02.png) · Root: Devices

Same device home; Living Room chip selected. Only Floor lamp and Living room TV cards, room-specific pin ordering. Subtle count 2 devices and empty breathing space. Plus and room management available. Bottom navigation unchanged.

Visible content: Room filters; device cards; observed state/freshness; explicit refresh; separate quick actions; three-root navigation.

| Action or outcome | Review destination |
|---|---|
| Open lamp | [C05](../output/prototypes/homeverb-window-light-v1/screens/C05.png) |
| Reorder cards | [D10](../output/prototypes/homeverb-window-light-v1/screens/D10.png) |
| All devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

### D03 — Find a device

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D03.png) · Nested screen

Search screen with back, focused search field containing lamp and clear icon. One matching card Floor lamp with room Living Room and state On. Recent searches AC and TV, unobtrusive. No fake keyboard occupying the screen. Search is local and has Cancel.

Visible content: Search devices / lamp; Searches local device names and rooms.; Matching fictional device cards: lamp.; Recent searches / AC · TV; Cancel

| Action or outcome | Review destination |
|---|---|
| Open result | [C05](../output/prototypes/homeverb-window-light-v1/screens/C05.png) |
| Clear and close | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

### D04 — First launch

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D04.png) · Root: Devices

HomeVerb header and three-tab navigation. Calm outlined house/device illustration, title Your home, together. Text Add a platform to find your appliances. Primary Add Platform and secondary Receive Authorization. Small text No HomeVerb account needed. No sign-in form, carousel, avatar or marketing hero.

Visible content: home / Your home, together / Add a platform to find your appliances.; Add Platform; Receive Authorization; No HomeVerb account needed.

| Action or outcome | Review destination |
|---|---|
| Add Platform | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |
| Receive Authorization | [H05](../output/prototypes/homeverb-window-light-v1/screens/H05.png) |

## Make room for your home

### R01 — Room management

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/R01.png) · Nested screen

Back header Rooms with plus icon. Reorderable list Living Room / 2 devices, Primary Bedroom / 2 devices, Bathroom / 1 device, Study / 0 devices. Each row has drag handle and overflow. Unassigned / 1 device below separated without delete. Hint Drag to reorder.

Visible content: Living Room / 2 devices  ≡ / Primary Bedroom / 2 devices  ≡ / Bathroom / 1 device  ≡ / Study / 0 devices  ≡; Unassigned / 1 device; Add Room; Drag to reorder. Use a row menu to rename or delete.

| Action or outcome | Review destination |
|---|---|
| Add Room | [R02](../output/prototypes/homeverb-window-light-v1/screens/R02.png) |
| Rename: invalid name | [R05](../output/prototypes/homeverb-window-light-v1/screens/R05.png) |
| Move devices | [R03](../output/prototypes/homeverb-window-light-v1/screens/R03.png) |
| Delete room | [R04](../output/prototypes/homeverb-window-light-v1/screens/R04.png) |

### R02 — Create a room

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/R02.png) · Nested screen · Sheet over R01

Room management dimmed behind a rounded bottom sheet Add Room. Labeled Room name field containing Kitchen, optional suggested chips Balcony, Nursery. Primary Save Room, secondary Cancel. No mandatory preset list.

Visible content: Room name / Kitchen; Balcony / Nursery; Save Room; Cancel

| Action or outcome | Review destination |
|---|---|
| See empty-room pattern | [D09](../output/prototypes/homeverb-window-light-v1/screens/D09.png) |
| Cancel | [R01](../output/prototypes/homeverb-window-light-v1/screens/R01.png) |

### R03 — Move devices

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/R03.png) · Nested screen

Back header Move to Room. Two selected devices Bedroom AC and Air sensor. Radio options Living Room, Primary Bedroom (selected), Bathroom, Study, Unassigned; Add Room link. Primary Move 2 Devices at bottom. Show organization only, no permission scope controls.

Visible content: Selected: Bedroom AC and Air sensor; Living Room / Primary Bedroom / Bathroom / Study / Unassigned; Add Room; Move 2 Devices; Rooms organize devices locally; access stays the same.

| Action or outcome | Review destination |
|---|---|
| Move selected devices | [D02](../output/prototypes/homeverb-window-light-v1/screens/D02.png) |
| Create another room | [R02](../output/prototypes/homeverb-window-light-v1/screens/R02.png) |

### R04 — Delete a room

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/R04.png) · Nested screen · Dialog over R01

Rooms dimmed behind centered confirmation Delete Study? Text Devices move to Unassigned. No devices or platform accounts are deleted. Buttons Cancel and Delete Room, destructive muted red. No scary full-screen warning.

Visible content: Devices move to Unassigned. No devices or platform accounts are deleted.; Delete Room; Cancel

| Action or outcome | Review destination |
|---|---|
| Delete Room | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |
| Cancel | [R01](../output/prototypes/homeverb-window-light-v1/screens/R01.png) |

## A place for every device

### D05 — Card actions

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D05.png) · Nested screen · Sheet over D01

Device home dimmed behind bottom sheet Floor lamp. Menu Rename, Move to Room, Pin to Top, Reorder Devices, Device Information. Clear icons, drag handle, close. No Delete Platform or credential controls.

Visible content: Rename / › / Move to Room / › / Pin to Top / › / Reorder Devices / › / Device Information / ›; Close

| Action or outcome | Review destination |
|---|---|
| Rename | [D07](../output/prototypes/homeverb-window-light-v1/screens/D07.png) |
| Move devices | [R03](../output/prototypes/homeverb-window-light-v1/screens/R03.png) |
| Reorder | [D10](../output/prototypes/homeverb-window-light-v1/screens/D10.png) |
| Device Information | [D06](../output/prototypes/homeverb-window-light-v1/screens/D06.png) |

### D06 — Device information

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D06.png) · Nested screen

Back header Device Information. Simple lamp icon; Floor lamp. Grouped rows Name / Floor lamp, Room / Living Room, Platform / Example Home, Model / Demo Light, Connection / Auto. Supported capabilities Power, Brightness, Color temperature. Local name and room have chevrons, no exposed serials or live network identifiers.

Visible content: lamp / Floor lamp / Living Room; Name / Floor lamp  › / Room / Living Room  › / Platform / Example Home / Model / Demo Light / Connection / Auto  ›; Supported: Power · Brightness · Color temperature

| Action or outcome | Review destination |
|---|---|
| Rename | [D07](../output/prototypes/homeverb-window-light-v1/screens/D07.png) |
| Move to Room | [R03](../output/prototypes/homeverb-window-light-v1/screens/R03.png) |
| Connection | [D08](../output/prototypes/homeverb-window-light-v1/screens/D08.png) |
| Platform | [P09](../output/prototypes/homeverb-window-light-v1/screens/P09.png) |

### D07 — Rename a device

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D07.png) · Nested screen · Sheet over D06

Device Information dimmed with bottom sheet Rename Device. Editable field Floor lamp, helper This name is saved on this device. Primary Save Name, Cancel. Small muted text Device access stays the same.

Visible content: Local name / Floor lamp; Saved on this installation. Device access stays the same.; Save Name; Cancel

| Action or outcome | Review destination |
|---|---|
| Save name | [D06](../output/prototypes/homeverb-window-light-v1/screens/D06.png) |
| Cancel | [D06](../output/prototypes/homeverb-window-light-v1/screens/D06.png) |

### D08 — Connection choice

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D08.png) · Nested screen

Back header Connection. Radio list Auto (selected) / Prefer local when available; Local network only / No cloud fallback; Platform cloud / Requires internet. Connection support applies to this device. Primary Save Preference. Keep user-facing text, no protocol jargon.

Visible content: Auto — prefer local when available / Local network only — no cloud fallback / Platform cloud — requires internet; Only routes supported by this device are available.; Save Preference

| Action or outcome | Review destination |
|---|---|
| Save preference | [D06](../output/prototypes/homeverb-window-light-v1/screens/D06.png) |

## Comfort, clearly controlled

### C01 — Climate control

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C01.png) · Nested screen

Back header Bedroom AC, info icon. Primary Bedroom subtitle. Large airy central 24 C target with minus and plus, terracotta arc; smaller Room 26 C / Updated just now. Power switch On. Rows Mode / Cool and Fan / Auto and Refresh State. No universal min/max text. Main focal area large and clean.

Visible content: Observed target 24°C; Cooling; room 26°C; explicit target stepper; timestamp. Example values only.; Power / On / Mode / Cool  › / Fan / Auto  ›; Refresh State; Device information · Recent operations

| Action or outcome | Review destination |
|---|---|
| Choose mode | [C02](../output/prototypes/homeverb-window-light-v1/screens/C02.png) |
| Submit target | [C03](../output/prototypes/homeverb-window-light-v1/screens/C03.png) |
| Device-information pattern | [D06](../output/prototypes/homeverb-window-light-v1/screens/D06.png) |
| Enter an invalid target | [C14](../output/prototypes/homeverb-window-light-v1/screens/C14.png) |

### C02 — Choose a mode

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C02.png) · Nested screen · Sheet over C01

Same climate screen dimmed with Choose Mode bottom sheet. Radio options Cool (selected), Heat, Dry, Fan only; Save Mode button. Only device-supported options. Keep currently observed temperature visible behind sheet.

Visible content: Cool / Heat / Dry / Fan only; Options come from this device’s capabilities.; Save Mode; Cancel

| Action or outcome | Review destination |
|---|---|
| Save mode | [C03](../output/prototypes/homeverb-window-light-v1/screens/C03.png) |
| Cancel | [C01](../output/prototypes/homeverb-window-light-v1/screens/C01.png) |

### C03 — Accepted, awaiting state

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C03.png) · Nested screen

Climate screen observed target 24 C remains visible. Distinct small pending card Requested 23 C with spinner and status Accepted; not yet confirmed. Supporting text Checking device state. Primary control temporarily subdued to prevent accidental repeated writes. No success tick yet.

Visible content: Observed target 24°C; Cooling; room 26°C; explicit target stepper; timestamp. Example values only.; Accepted; not yet confirmed / Requested 23°C. Observed target remains 24°C.; Waiting for an updated reading; The pending request has not replaced the last observation.

| Action or outcome | Review destination |
|---|---|
| Confirmation unavailable | [C04](../output/prototypes/homeverb-window-light-v1/screens/C04.png) |
| View operations | [S05](../output/prototypes/homeverb-window-light-v1/screens/S05.png) |

### C04 — Unknown outcome

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C04.png) · Nested screen

Climate screen with observed target 24 C and Last read 2 min ago. Amber rounded status card Result unknown / The change may have happened. Primary Refresh State, secondary View Details. No Retry Command, no automatic reroute button. Calm useful tone.

Visible content: Observed target 24°C; Cooling; room 26°C; explicit target stepper; timestamp. Example values only.; Result unknown / The change may have happened. Last read: 2 min ago.; Refresh State; View Details

| Action or outcome | Review destination |
|---|---|
| Refresh state | [C01](../output/prototypes/homeverb-window-light-v1/screens/C01.png) |
| View operations | [S05](../output/prototypes/homeverb-window-light-v1/screens/S05.png) |

## Controls that fit the appliance

### C05 — Lighting

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C05.png) · Nested screen

Back Floor lamp. Living Room. Power On. Large simple warm lamp illustration. Brightness 65% slider and Color temperature 3000 K slider, subtle warm-to-cool track. Updated just now and Refresh. No temperature claims for all brands.

Visible content: lamp / On / Living Room; Brightness / 65%; Color temperature / 3000 K; Power / On; Updated just now · Limits come from this device.; Refresh State

| Action or outcome | Review destination |
|---|---|
| Device information | [D06](../output/prototypes/homeverb-window-light-v1/screens/D06.png) |
| View operation evidence | [S06](../output/prototypes/homeverb-window-light-v1/screens/S06.png) |

### C06 — Fan

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C06.png) · Nested screen

Back Bedroom fan. Primary Bedroom. Simple fan icon, Power On, Speed 3 stepper with device-supported marks 1 through 5. Oscillation On switch, preset Normal selection. No unrelated temperature controls.

Visible content: fan / Power on / Primary Bedroom; Speed / 3; 1 / 2 / 3 / 4 / 5; Oscillation / On / Preset / Normal  ›; Refresh State

| Action or outcome | Review destination |
|---|---|
| Back to devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

### C07 — Media

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C07.png) · Nested screen

Back Living room TV. Large TV icon, state Paused, source HDMI 1 with selector. Accessible play/pause control, Volume 25 slider, Muted Off toggle, Power On. No streaming artwork or cloud subscriptions.

Visible content: tv / Paused / Living Room; Source / HDMI 1  › / Power / On / Muted / Off; Play; Volume / 25; Refresh State

| Action or outcome | Review destination |
|---|---|
| Pair this TV | [P06](../output/prototypes/homeverb-window-light-v1/screens/P06.png) |
| Back to devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

### C08 — Components and sensors

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C08.png) · Nested screen

Back Refrigerator. Two large section cards Fridge / Current 4 C / Target 4 C with stepper and Freezer / Current -18 C / Target -18 C with stepper. Secondary read-only Door / Closed. Freshness visible. Components stay within one physical device, not two separate fridge tiles.

Visible content: Fridge / Current / 4°C / Target / −  4°C  +; Freezer / Current / −18°C / Target / −  −18°C  +; Door / Closed / Updated / Just now; One appliance, separate components. Each uses its own supported limits.; Refresh State

| Action or outcome | Review destination |
|---|---|
| Back to devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

## Know what happened

### C09 — Appliance cycle

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C09.png) · Nested screen

Back Washing machine, Bathroom. Large simple washer icon. State Ready. Program selector Cotton, duration 58 min as device estimate. Primary Start Cycle. No background automation or scheduling controls.

Visible content: washer / Ready / Bathroom; Program / Cotton  › / Estimated duration / 58 min; Start Cycle; Estimate reported by this appliance.

| Action or outcome | Review destination |
|---|---|
| Start Cycle | [C10](../output/prototypes/homeverb-window-light-v1/screens/C10.png) |

### C10 — Confirm a cycle

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C10.png) · Nested screen · Dialog over C09

Washer screen dimmed behind clear confirmation Start Cotton cycle? Device Washing machine / Bathroom. Text The appliance continues if you close HomeVerb. Primary Start Cycle, secondary Cancel. Specific action and target, no generic confirmed checkbox.

Visible content: Device / Washing machine / Room / Bathroom; The appliance continues if you close HomeVerb.; Start Cycle; Cancel

| Action or outcome | Review destination |
|---|---|
| Confirm start | [C11](../output/prototypes/homeverb-window-light-v1/screens/C11.png) |
| Cancel | [C09](../output/prototypes/homeverb-window-light-v1/screens/C09.png) |

### C11 — Verified state

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C11.png) · Nested screen

Washer device page with terracotta checked result State verified. Current state Running / Program Cotton / Updated 09:42. Small action View Evidence. Pause and Stop are distinct device-supported actions. No claim of guaranteed causality or exactly-once.

Visible content: State verified / Current state matches the requested state.; washer / Running / Cotton · Updated 09:42; View Evidence; Pause / Supported action  › / Stop / Supported action  ›; State matching alone does not prove causality.

| Action or outcome | Review destination |
|---|---|
| View operations | [S05](../output/prototypes/homeverb-window-light-v1/screens/S05.png) |
| Back to devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

### C12 — Partial result

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C12.png) · Nested screen

Back Operation Results. Header 3 devices / Partially completed. Separate rows Floor lamp / Verified / On, Bedroom AC / Accepted / Waiting for state, TV / Failed / Permission denied. View Details per row and Done at bottom. No global success or Roll Back button.

Visible content: Partially completed / 3 devices · Outcomes remain separate.; Floor lamp / Verified · On / Bedroom AC / Accepted · Awaiting state / Living room TV / Failed · Permission denied; Open an item for its evidence. Completed actions are not rolled back.; Done

| Action or outcome | Review destination |
|---|---|
| View operation details | [S06](../output/prototypes/homeverb-window-light-v1/screens/S06.png) |
| Done | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

## Bring appliances together

### A01 — Choose an add method

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/A01.png) · Nested screen

Back Add Device. Two large concise option cards From a Platform / Choose a connected account and Nearby Devices / Discover on your local network. Link Add Another Platform. No gateway address, backend setup, or HomeVerb login.

Visible content: From a Platform / Account / Choose a connected account  ›; Nearby Devices / Discovery / Use your local network  ›; Add Another Platform

| Action or outcome | Review destination |
|---|---|
| From a Platform | [P01](../output/prototypes/homeverb-window-light-v1/screens/P01.png) |
| Nearby Devices | [A02](../output/prototypes/homeverb-window-light-v1/screens/A02.png) |
| Add Another Platform | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |

### A02 — Local permission

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/A02.png) · Nested screen · Sheet over A01

Add Device background with pre-permission panel Find devices nearby. Text Local network access lets HomeVerb discover compatible appliances while you use this screen. Primary Continue; secondary Not Now. Small system-style local network dialog preview could be avoided; only this app explanation.

Visible content: Local network access lets HomeVerb discover compatible appliances while you use this screen.; Continue; Not Now

| Action or outcome | Review destination |
|---|---|
| Allow local discovery | [A03](../output/prototypes/homeverb-window-light-v1/screens/A03.png) |
| Deny permission | [A08](../output/prototypes/homeverb-window-light-v1/screens/A08.png) |
| Not Now | [A01](../output/prototypes/homeverb-window-light-v1/screens/A01.png) |

### A03 — Discovering

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/A03.png) · Nested screen

Back Nearby Devices. Simple terracotta scanning ring with device icon, Searching nearby / 6 seconds remaining. Already found Floor lamp and Living room TV appear as selectable rows; remaining scanning indicator. Cancel action visible. No endless scan or background toggle.

Visible content: search / Searching nearby / 6 seconds remaining; Floor lamp / Found / Living room TV / Found; Searching on your local network; Cancel

| Action or outcome | Review destination |
|---|---|
| Discovery completes | [A04](../output/prototypes/homeverb-window-light-v1/screens/A04.png) |
| No results before deadline | [A07](../output/prototypes/homeverb-window-light-v1/screens/A07.png) |
| Cancel | [A01](../output/prototypes/homeverb-window-light-v1/screens/A01.png) |

### A04 — Select discovered devices

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/A04.png) · Nested screen

Back Found Devices. Three checkbox rows Floor lamp / Local network (selected), Living room TV / Pairing required (selected), Bedroom AC / Possible duplicate (not selected). Room dropdown Unassigned. Primary Add 2 Devices. Secondary Review Duplicate. Clear count and no automatic merging.

Visible content: Floor lamp · Local network / Living room TV · Pairing required / Bedroom AC · Possible duplicate; Room / Unassigned  ›; Add 2 Devices; Review Duplicate

| Action or outcome | Review destination |
|---|---|
| Add selected devices | [A06](../output/prototypes/homeverb-window-light-v1/screens/A06.png) |
| Review Duplicate | [A05](../output/prototypes/homeverb-window-light-v1/screens/A05.png) |
| Choose room | [R03](../output/prototypes/homeverb-window-light-v1/screens/R03.png) |

## Finish adding, or recover

### A05 — Resolve a possible duplicate

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/A05.png) · Nested screen

Back Possible Duplicate. Two comparison cards Existing Bedroom AC and Found Bedroom AC, each with masked evidence labels Model and Connection, no raw identifiers. Text Names alone do not prove a match. Buttons Keep Separate and Confirm Same Device. No preselected merge.

Visible content: Existing Bedroom AC / Model / Demo AC / Connection / Saved connection; Found Bedroom AC / Model / Demo AC / Connection / Discovered connection; Names alone do not prove a match.; Keep Separate; Confirm Same Device

| Action or outcome | Review destination |
|---|---|
| Keep Separate | [A04](../output/prototypes/homeverb-window-light-v1/screens/A04.png) |
| Confirm Same Device | [A04](../output/prototypes/homeverb-window-light-v1/screens/A04.png) |

### A06 — Devices added

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/A06.png) · Nested screen

Back Devices Added. Terracotta check and title 2 devices added. Rows Floor lamp / Living Room (editable room dropdown) and TV / Living Room (editable). Primary Open Living Room, secondary Add More. Fresh state is not fabricated.

Visible content: 2 devices added / You can change their local rooms.; Floor lamp / Living Room  › / Living room TV / Living Room  ›; Open Living Room; Add More; Device state is read only when requested.

| Action or outcome | Review destination |
|---|---|
| Open room | [D02](../output/prototypes/homeverb-window-light-v1/screens/D02.png) |
| Add More | [A01](../output/prototypes/homeverb-window-light-v1/screens/A01.png) |

### A07 — Nothing found

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/A07.png) · Nested screen

Back Nearby Devices. Minimal disconnected-device line illustration. Title No devices found. Helpful checks Same reachable network and Device is ready to pair. Primary Search Again, secondary Choose a Platform. No running spinner after timeout.

Visible content: search / No devices found / The search has ended.; Check / Same reachable network / Check / Device is ready to pair; Search Again; Choose a Platform

| Action or outcome | Review destination |
|---|---|
| Search Again | [A03](../output/prototypes/homeverb-window-light-v1/screens/A03.png) |
| Choose a Platform | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |

### A08 — Permission denied

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/A08.png) · Nested screen

Back Nearby Devices. Restrained permission icon. Title Local access is off. Text Enable local network access in system settings to discover nearby devices. Primary Open System Settings, secondary Back. No silent switch to a cloud route.

Visible content: Local access is off / Enable local network access in system settings to discover nearby devices.; Open System Settings; Back

| Action or outcome | Review destination |
|---|---|
| Return after system permission change | [A02](../output/prototypes/homeverb-window-light-v1/screens/A02.png) |
| Back | [A01](../output/prototypes/homeverb-window-light-v1/screens/A01.png) |

## Connect your appliance platforms

### P01 — Platform accounts

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P01.png) · Root: Platforms

Root header Smart Appliance Platforms, plus icon. Cards Example Home / Home account / 6 devices / Checked today, Example Home / Studio account / 2 devices / Sign-in needed (amber), Example TV / Living room / 1 device / Checked yesterday. Generic letter icons. Three bottom tabs with Platforms selected. No Data Sources wording.

Visible content: Example Home / Home account / 6 devices · Checked today; Example Home / Studio account / 2 devices · Sign-in needed; Example TV / Living room / 1 device · Checked yesterday; Add Platform; Illustrative accounts. Compatibility is model-specific.

| Action or outcome | Review destination |
|---|---|
| Add platform | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |
| Account details | [P09](../output/prototypes/homeverb-window-light-v1/screens/P09.png) |
| Reauthorize expired account | [P10](../output/prototypes/homeverb-window-light-v1/screens/P10.png) |
| Devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |
| Settings | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### P02 — Select a platform

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P02.png) · Nested screen

Back Add Platform, Search platforms field. List Example Home / Browser authorization, Example TV / Device pairing, Example Climate / Not available yet with disabled badge. Regions and compatibility details available. Tiny footer Illustrative platforms in this design, not actual compatibility claims.

Visible content: Search platforms / ; Example Home / Browser authorization  › / Example TV / Device pairing  › / Example Climate / Not available yet; Regions and compatibility; Fictional platform names; not support claims.

| Action or outcome | Review destination |
|---|---|
| Example Home | [P03](../output/prototypes/homeverb-window-light-v1/screens/P03.png) |
| Example TV | [P06](../output/prototypes/homeverb-window-light-v1/screens/P06.png) |
| Unavailable platform | [P13](../output/prototypes/homeverb-window-light-v1/screens/P13.png) |

### P03 — Choose region and account

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P03.png) · Nested screen

Back Connect Example Home. Region selector Europe, local Account name field Home account. Concise permission preview Read devices and Control supported appliances. Primary Continue to Authorization; Cancel. No HomeVerb signup or password field.

Visible content: Region / Europe  ›; Account name / Home account; Requested access / Read / Device state / Control / Supported appliances; Continue to Authorization; Cancel

| Action or outcome | Review destination |
|---|---|
| Continue to Authorization | [P04](../output/prototypes/homeverb-window-light-v1/screens/P04.png) |
| Cancel | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |

### P04 — Native browser consent

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P04.png) · Nested screen

System browser-style view example.invalid (fictional) and close icon. Example Home title Allow HomeVerb access? Read device state and Control supported devices listed. Masked account h***@example.invalid. Allow and Cancel. Visibly a third-party authorization page, not HomeVerb account registration.

Visible content: platform / Example Home / Third-party authorization; Allow HomeVerb access?; Account / h***@example.invalid / Permission / Read device state / Permission / Control supported devices; Allow; Cancel; Provider-owned browser page; its appearance may differ.

| Action or outcome | Review destination |
|---|---|
| Allow | [P08](../output/prototypes/homeverb-window-light-v1/screens/P08.png) |
| Code challenge where supported | [P05](../output/prototypes/homeverb-window-light-v1/screens/P05.png) |
| Cancel | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |

## Authorization, step by step

### P05 — Verification code

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P05.png) · Nested screen

Back Verify Example Home. Text Enter the code from your platform, masked destination ***42. Six clear code boxes with fictional 4 8 2 1 empty empty. Resend in 00:28. Continue disabled until complete; Cancel. No real phone number or credential.

Visible content: Enter the code from your platform. Sent to ***42.; 4  8  2  1  _  _; Resend in 00:28; Continue; Cancel

| Action or outcome | Review destination |
|---|---|
| Complete valid code | [P08](../output/prototypes/homeverb-window-light-v1/screens/P08.png) |
| Code expires | [P07](../output/prototypes/homeverb-window-light-v1/screens/P07.png) |
| Cancel | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |

### P06 — Approve on the appliance

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P06.png) · Nested screen

Back Pair Living room TV. Simple TV illustration displaying code 482 196 (fictional). Text Confirm this code on your TV. Waiting for approval / 01:20 remaining and Cancel. No universal QR login claim.

Visible content: tv / 482 196 / Confirm this fictional code on your TV.; Waiting for approval / 01:20 remaining; Cancel

| Action or outcome | Review destination |
|---|---|
| Approve on TV | [P08](../output/prototypes/homeverb-window-light-v1/screens/P08.png) |
| Deadline expires | [P07](../output/prototypes/homeverb-window-light-v1/screens/P07.png) |
| Cancel | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |

### P07 — Authorization expired

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P07.png) · Nested screen

Back Connect Example Home. Calm amber clock icon, Authorization expired / Start again to request a new authorization. Primary Start Again, secondary Cancel. No endless automatic login loop.

Visible content: Authorization expired / Start again to request a new authorization.; Start Again; Cancel

| Action or outcome | Review destination |
|---|---|
| Start Again | [P03](../output/prototypes/homeverb-window-light-v1/screens/P03.png) |
| Cancel | [P01](../output/prototypes/homeverb-window-light-v1/screens/P01.png) |

### P08 — Platform connected

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P08.png) · Nested screen

The rendered image is the earlier generic example: Authorization saved securely, Find Devices, and Done. For the required Meiju journey, authorization instead continues automatically into bounded account-device retrieval. Show Loading Devices with Cancel; then the actual supported/unsupported counts and Open Devices. Do not require Find Devices or individual device selection. Preserve an authorization-success message when retrieval fails, offer Retry Retrieval, and distinguish empty from partial results. New supported cards start Unassigned; reauthorization preserves existing organization. No raw token display or invented fresh device state.

Required content, pending visual update: Authorization saved securely; Loading Devices; Cancel; retrieved device counts and unsupported reasons; Open Devices; optional room assignment. Error/partial variants offer Retry Retrieval and View Available Devices; a confirmed empty account explains that no devices were returned.

| Action or outcome | Review destination |
|---|---|
| Authorization saved | P08 retrieval variant, started automatically within the foreground connection flow |
| Retrieval completed / Open Devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png), using actual retrieved cards; empty variant when no devices are returned |
| Retrieval failed or partial / Retry Retrieval | P08 retrieval variant; explicit bounded retry using the saved session when valid |
| Partial result / View Available Devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png), with incomplete retrieval disclosed |
| Ambiguous identity | [A05](../output/prototypes/homeverb-window-light-v1/screens/A05.png), returning to the initiating account flow |
| Assign local rooms | [R03](../output/prototypes/homeverb-window-light-v1/screens/R03.png), without gating retrieval or changing permissions |
| Cancel retrieval | [P09](../output/prototypes/homeverb-window-light-v1/screens/P09.png), preserving securely saved authorization and known results without automatic resumption |
| Done | [P01](../output/prototypes/homeverb-window-light-v1/screens/P01.png) |

## Manage access deliberately

### P09 — Platform details

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P09.png) · Nested screen

Back Example Home. Home account, masked h***@example.invalid, Europe. Rows Devices / 6, Permissions / Read and control, Last checked / Today 09:40. Actions Check Connection, Reauthorize. Secondary Remove From This Device. Remote Revoke Access action distinct at bottom with provider-supported note.

Visible content: Home account · h***@example.invalid; Region / Europe / Devices / 6 / Permissions / Read and control / Last checked / Today 09:40; Check Connection; Reauthorize; Remove From This Device; Revoke Access · if supported

| Action or outcome | Review destination |
|---|---|
| Expired check result | [P10](../output/prototypes/homeverb-window-light-v1/screens/P10.png) |
| Reauthorize | [P03](../output/prototypes/homeverb-window-light-v1/screens/P03.png) |
| Remove locally | [P11](../output/prototypes/homeverb-window-light-v1/screens/P11.png) |
| Revoke grant | [P12](../output/prototypes/homeverb-window-light-v1/screens/P12.png) |

### P10 — Reconnect needed

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P10.png) · Nested screen

Same account detail with amber Sign-in needed / Your platform authorization has expired. Cached device count remains labelled Last known. Primary Reauthorize. Secondary View Devices and Remove From This Device. No other account auto-selection.

Visible content: Sign-in needed / Your platform authorization has expired.; Devices / 6 · Last known; Reauthorize; View Devices; Remove From This Device

| Action or outcome | Review destination |
|---|---|
| Reauthorize | [P03](../output/prototypes/homeverb-window-light-v1/screens/P03.png) |
| View cached devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |
| Remove locally | [P11](../output/prototypes/homeverb-window-light-v1/screens/P11.png) |

### P11 — Remove local account

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P11.png) · Nested screen · Dialog over P09

Account detail dimmed, confirmation Remove from this device? Text Local authorization and associated cached data will be removed. Access on other devices is not revoked. Cancel and Remove Locally. No false global logout claim.

Visible content: Local authorization and associated cached data will be removed. Access on other devices is not revoked.; Remove Locally; Cancel

| Action or outcome | Review destination |
|---|---|
| Remove Locally | [P01](../output/prototypes/homeverb-window-light-v1/screens/P01.png) |
| Cancel | [P09](../output/prototypes/homeverb-window-light-v1/screens/P09.png) |

### P12 — Revoke provider access

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P12.png) · Nested screen · Dialog over P09

Account detail dimmed, confirmation Revoke platform access? Text Example Home supports revoking this grant. Other installations using this grant may lose access. Cancel and Revoke Access. An explicit provider action, not a local delete.

Visible content: Example Home supports revoking this grant. Other installations using it may lose access.; Revoke Access; Cancel

| Action or outcome | Review destination |
|---|---|
| Revoke Access | [P01](../output/prototypes/homeverb-window-light-v1/screens/P01.png) |
| Cancel | [P09](../output/prototypes/homeverb-window-light-v1/screens/P09.png) |

## Quiet, practical settings

### S01 — Settings home

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S01.png) · Root: Settings

Root Settings. No avatar or profile. Groups: Home / Rooms; Authorization Sharing / Send Authorization and Receive Authorization; Preferences / Appearance, This Device, Connections; Data & Support / Recent Operations, Local Data, Diagnostics, About. Three bottom tabs, Settings selected. Spacious but all key groups visible.

Visible content: HOME; Rooms / ›; AUTHORIZATION SHARING; Send Authorization / › / Receive Authorization / ›; PREFERENCES; Appearance / Window Light  › / This Device / › / Connections / Auto  ›; DATA & SUPPORT; Recent Operations / › / Local Data / › / Diagnostics / › / About / ›

| Action or outcome | Review destination |
|---|---|
| Rooms | [R01](../output/prototypes/homeverb-window-light-v1/screens/R01.png) |
| Send Authorization | [H01](../output/prototypes/homeverb-window-light-v1/screens/H01.png) |
| Receive Authorization | [H05](../output/prototypes/homeverb-window-light-v1/screens/H05.png) |
| Appearance | [S02](../output/prototypes/homeverb-window-light-v1/screens/S02.png) |
| This Device | [S10](../output/prototypes/homeverb-window-light-v1/screens/S10.png) |
| Connections | [S03](../output/prototypes/homeverb-window-light-v1/screens/S03.png) |
| Recent Operations | [S05](../output/prototypes/homeverb-window-light-v1/screens/S05.png) |
| Local Data | [S04](../output/prototypes/homeverb-window-light-v1/screens/S04.png) |
| Diagnostics | [S07](../output/prototypes/homeverb-window-light-v1/screens/S07.png) |
| About | [S08](../output/prototypes/homeverb-window-light-v1/screens/S08.png) |

### S02 — Appearance and skin

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S02.png) · Nested screen

Settings > Appearance contains exactly one skin option, Window Light, marked Current. A preview opens S11. No alternate skins, download store, custom wallpaper, dark or system-mode selector ships in this release. Text follows system accessibility settings. The stable local skin identifier is window_light; later reviewed skins can extend the registry.

Visible content: Skin; One Window Light tile, marked Current; stable skin ID window_light.; Window Light is the only included skin and is already selected.; Preview Skin; Text size and accessibility; Text follows system accessibility settings.

| Action or outcome | Review destination |
|---|---|
| Preview Window Light | [S11](../output/prototypes/homeverb-window-light-v1/screens/S11.png) |
| Back | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |
| Larger text layout | [D12](../output/prototypes/homeverb-window-light-v1/screens/D12.png) |

### S03 — Connection preferences

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S03.png) · Nested screen

Back Connections. Default route Auto selected, Local network only and Platform cloud options with short explanations. Local network permission / Allowed with Open Settings action. Hint A device-specific choice takes priority. No daemon, keep-alive, or background scan switch.

Visible content: Auto — prefer local when available / Local network only — no cloud fallback / Platform cloud — requires internet; Local network permission / Allowed; Open Settings; A device-specific preference takes priority. No background scanning.

| Action or outcome | Review destination |
|---|---|
| Back | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### S04 — Local data

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S04.png) · Nested screen

Back Local Data. Sections Device organization / Rooms and names; Cached device state / Last read snapshots; Recent operations / Latest 100 records. Clear Cached State and Clear Recent Operations distinct actions. Full Remove Local Data action at bottom, privacy note Authorizations are stored securely. No export token option.

Visible content: Device organization / Rooms and names / Cached device state / Last-read snapshots / Recent operations / Latest 100 records; Clear Cached State; Clear Recent Operations; Remove Local Data; Authorizations are held in secure storage. Local removal does not revoke remote access.

| Action or outcome | Review destination |
|---|---|
| Confirm removal | [S09](../output/prototypes/homeverb-window-light-v1/screens/S09.png) |
| Back | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

## Send access nearby

### H01 — Choose authorization

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H01.png) · Nested screen

Back Send Authorization. Eligible Home account / Example Home / Full account access selected. Studio account / Example Home / Sign-in needed disabled. Example TV / Device-bound / Cannot share disabled. Primary Find Recipient. Short permissions note Actual access depends on the platform.

Visible content: Home account · Full account access / Studio account · Sign-in needed / Example TV · Device-bound; Only the first account is eligible. Actual access depends on the platform.; Find Recipient

| Action or outcome | Review destination |
|---|---|
| Find Recipient | [H02](../output/prototypes/homeverb-window-light-v1/screens/H02.png) |
| Cannot share | [H11](../output/prototypes/homeverb-window-light-v1/screens/H11.png) |

### H02 — Find a recipient

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H02.png) · Nested screen

Back Select Recipient. This device / Home phone. Nearby receiver card Alex phone / Ready to receive, simple terracotta device icon; another Morgan tablet / Ready to receive. Select Alex phone. Small text Ask the recipient to open Receive Authorization. Scanning countdown and Cancel.

Visible content: This device: Home phone; Alex phone / Ready to receive / Morgan tablet / Ready to receive; Ask the recipient to open Receive Authorization.; Scanning · 01:20 remaining; Cancel

| Action or outcome | Review destination |
|---|---|
| Choose Alex phone | [H03](../output/prototypes/homeverb-window-light-v1/screens/H03.png) |
| No peers found | [H13](../output/prototypes/homeverb-window-light-v1/screens/H13.png) |
| Cancel | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### H03 — Verify the recipient

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H03.png) · Nested screen

Back Verify Recipient. Alex phone and Home phone simple device icons linked by a line. Large pairing code 482 196. Text Compare this code on both devices. Primary Codes Match and secondary Codes Do Not Match. No credentials sent yet.

Visible content: Home phone → Alex phone; 482 196; Compare this code on both devices. No authorization has been sent.; Codes Match; Codes Do Not Match

| Action or outcome | Review destination |
|---|---|
| Codes Match | [H04](../output/prototypes/homeverb-window-light-v1/screens/H04.png) |
| Codes Do Not Match | [H14](../output/prototypes/homeverb-window-light-v1/screens/H14.png) |

### H04 — Review before sending

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H04.png) · Nested screen

Back Review Authorization. Recipient Alex phone; Platform Example Home; Account Home account. Scope card Read and control all 6 devices. Text This grant cannot be limited to selected devices. Expiry / Set by platform. Revocation / Platform account settings. Primary Send Authorization, Cancel. No fake custom expiry or per-device restriction toggles.

Visible content: Recipient / Alex phone / Platform / Example Home / Account / Home account / Scope / Read and control all 6 devices / Expiry / Not provided by platform / Revocation / Platform account settings; This grant cannot be limited to selected devices.; Send Authorization; Cancel

| Action or outcome | Review destination |
|---|---|
| Send Authorization | [H09](../output/prototypes/homeverb-window-light-v1/screens/H09.png) |
| Cancel | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

## Receive and connect independently

### H05 — Ready to receive

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H05.png) · Nested screen

Back Receive Authorization. Large calm terracotta receiver ring. This device / Alex phone; Visible for 01:48. Text Keep this screen open on the same local network. Primary Stop Receiving. No always-on toggle or sharing bottom tab.

Visible content: phone / Ready to receive / Alex phone · Visible for 01:48; Keep this screen open on the same reachable local network.; Stop Receiving

| Action or outcome | Review destination |
|---|---|
| Incoming request | [H06](../output/prototypes/homeverb-window-light-v1/screens/H06.png) |
| Stop Receiving | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |
| Leave foreground | [H15](../output/prototypes/homeverb-window-light-v1/screens/H15.png) |

### H06 — Review incoming access

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H06.png) · Nested screen

Back Incoming Authorization. From Home phone. Pairing code 482 196, compare-code instruction. Example Home / Home account / Read and control all 6 devices. Primary Accept Authorization, secondary Decline. Clear sender, actual permissions and local secure storage statement.

Visible content: From Home phone; 482 196; Compare this code with the sender.; Platform / Example Home / Account / Home account / Scope / Read and control all 6 devices; Import requires system secure storage.; Accept Authorization; Decline

| Action or outcome | Review destination |
|---|---|
| Accept | [H08](../output/prototypes/homeverb-window-light-v1/screens/H08.png) |
| Existing account found | [H07](../output/prototypes/homeverb-window-light-v1/screens/H07.png) |
| Storage unavailable | [H10](../output/prototypes/homeverb-window-light-v1/screens/H10.png) |
| Decline | [H05](../output/prototypes/homeverb-window-light-v1/screens/H05.png) |
| Imported grant rejected by platform | [H16](../output/prototypes/homeverb-window-light-v1/screens/H16.png) |

### H07 — Existing account conflict

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H07.png) · Nested screen

Back Account Already Exists. Existing Home account / Example Home and incoming Home account / Example Home shown. Text Your saved authorization will not be overwritten automatically. Actions Keep Existing and Replace After Checking. No silent merge, no overwrite by default.

Visible content: Existing authorization / Account / Home account · Example Home; Incoming authorization / Account / Home account · Example Home; Your saved authorization will not be overwritten automatically.; Keep Existing; Replace After Checking

| Action or outcome | Review destination |
|---|---|
| Keep Existing | [P09](../output/prototypes/homeverb-window-light-v1/screens/P09.png) |
| Replace After Checking | [H08](../output/prototypes/homeverb-window-light-v1/screens/H08.png) |

### H08 — Import result

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H08.png) · Nested screen

Back Authorization Received. Checklist Transferred (check), Imported securely (check), Authorization validated (check). Example Home / Home account. Text This phone now connects independently. Primary Choose Devices, secondary Done. Room assignment happens next; not a permission restriction.

Visible content: Authorization validated / This phone now connects independently.; Transferred / Complete / Imported securely / Complete / Authorization check / Validated; Choose Devices; Done; Room assignment organizes devices; it does not restrict this grant.

| Action or outcome | Review destination |
|---|---|
| Choose Devices | [A04](../output/prototypes/homeverb-window-light-v1/screens/A04.png) |
| Done | [P01](../output/prototypes/homeverb-window-light-v1/screens/P01.png) |

## Make sharing outcomes explicit

### H09 — Transferring

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H09.png) · Nested screen

Back Sending Authorization. Home phone to Alex phone, simple icons. Linear progress 62%, text Encrypted transfer. Stages Transfer active, Import waiting, Check waiting. Cancel Transfer button and Keep both apps open text. No credential strings or decorative charts.

Visible content: Home phone → Alex phone; Encrypted transfer · 62%; Transfer / Active / Secure import / Waiting / Authorization check / Waiting; Keep both apps open.; Cancel Transfer

| Action or outcome | Review destination |
|---|---|
| Receiver confirms import | [H08](../output/prototypes/homeverb-window-light-v1/screens/H08.png) |
| Receipt missing | [H12](../output/prototypes/homeverb-window-light-v1/screens/H12.png) |
| Interrupted or cancelled | [H15](../output/prototypes/homeverb-window-light-v1/screens/H15.png) |

### H10 — Secure storage unavailable

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H10.png) · Nested screen

Back Receive Authorization. Restrained lock icon. Title Secure storage unavailable. Text Authorization was not imported. Enable secure storage before trying again. Primary Open Settings, secondary Cancel. No Save as File or plaintext fallback.

Visible content: Secure storage unavailable / Authorization was not imported. Enable secure storage before trying again.; Open Settings; Cancel

| Action or outcome | Review destination |
|---|---|
| Cancel | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### H11 — Cannot share this account

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H11.png) · Nested screen

Back Send Authorization. Example TV / Living room. Title This authorization cannot be shared. Text It is tied to the original device. The recipient is instructed to open Platforms and complete pairing on their own device. Primary Back to Accounts. No remote pairing, bypass or extraction action.

Visible content: This account cannot be shared / Example TV · Living room; Authorization is tied to the original device. The recipient must open Platforms and pair on their own device.; Back to Accounts

| Action or outcome | Review destination |
|---|---|
| Review recipient pairing pattern | [P06](../output/prototypes/homeverb-window-light-v1/screens/P06.png) |
| Back to Accounts | [H01](../output/prototypes/homeverb-window-light-v1/screens/H01.png) |

### H12 — Receipt not confirmed

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H12.png) · Nested screen

Back Transfer Result. Amber outlined status Unconfirmed. Text The recipient may have imported the authorization. Ask them to check their result. Timeline Sent / Import confirmation unavailable / Authorization check unavailable. Primary Done, secondary View Local Record. No Retry or automatic resend.

Visible content: Unconfirmed / The recipient may have imported the authorization. Ask them to check their result.; Sent / Complete / Import confirmation / Unavailable / Authorization check / Unavailable; Done; View Local Record

| Action or outcome | Review destination |
|---|---|
| View Local Record | [S05](../output/prototypes/homeverb-window-light-v1/screens/S05.png) |
| Done | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

## Understand and recover

### S05 — Recent operations

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S05.png) · Nested screen

Back Recent Operations. Filter chips All, Devices, Sharing. Rows Bedroom AC / Result unknown / 09:42; Floor lamp / Verified / 09:40; TV / Failed / Permission denied; Home account sharing / Imported / 09:38. Distinct status text and subtle icon colors, chevrons for details.

Visible content: All / Devices / Sharing; Bedroom AC / Unknown · 09:42 / Floor lamp / Verified · 09:40 / Living room TV / Failed · Permission denied / Authorization sharing / Imported · 09:38; Select a record to inspect the evidence.

| Action or outcome | Review destination |
|---|---|
| Read evidence | [S06](../output/prototypes/homeverb-window-light-v1/screens/S06.png) |
| Partial batch result | [C12](../output/prototypes/homeverb-window-light-v1/screens/C12.png) |
| Unconfirmed transfer | [H12](../output/prototypes/homeverb-window-light-v1/screens/H12.png) |

### S06 — Operation evidence

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S06.png) · Nested screen

Back Operation Details. Floor lamp / Set power On. Status Verified. Requested On at 09:40:01, Observed On at 09:40:02, Source Local network. Explanation State matches the request; this alone does not prove causality. Refresh State and Done. No raw secrets or network identifiers.

Visible content: Verified / Floor lamp · Set power On; Requested / On · 09:40:01 / Observed / On · 09:40:02 / Source / Local network; State matches the request; this alone does not prove causality.; Refresh State; Done

| Action or outcome | Review destination |
|---|---|
| Back to operations | [S05](../output/prototypes/homeverb-window-light-v1/screens/S05.png) |

### S07 — Diagnostic preview

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S07.png) · Nested screen

Diagnostics preview uses the current allowlisted Core report: version, generation time, local profile counts, route counts, retention limits and receipt summaries. The pictured summary is abbreviated; production exposes every exported field before saving exactly the previewed report. No raw account or device identifiers, automatic upload or credential payloads.

Visible content: App version / 0.2.0 · Illustrative / Generated / Today 09:42 / Local scope / This profile / Devices / rooms / 4 / 3 / LAN / cloud routes / 4 / 2 / Receipt / attempt limits / 100 / 32; Latest receipt / Outcome / Unknown / Error / WRITE_OUTCOME_UNKNOWN / Phase / Possibly sent · LAN / Attempts omitted / 0; Review before saving. Account and device identifiers are excluded.; Export Redacted Report; Cancel

| Action or outcome | Review destination |
|---|---|
| Cancel preview | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### S08 — About and compatibility

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S08.png) · Nested screen

Back About. HomeVerb wordmark, One language for your appliances. Design preview label. Rows Version, Compatibility, Privacy, Licenses. Compatibility note Support varies by platform, model and region. No claims that preview platforms are verified, no login or subscription upsell.

Visible content: home / HomeVerb / One language for your appliances.; Design preview; Version / Preview / Compatibility / › / Privacy / › / Licenses / ›; Support varies by platform, model and region.

| Action or outcome | Review destination |
|---|---|
| Compatibility | [S12](../output/prototypes/homeverb-window-light-v1/screens/S12.png) |
| Back | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

## Recover without repeating secrets

### H13 — No recipient found

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H13.png) · Nested screen

Back Select Recipient. Simple two phone outline, No nearby recipients. Checks Same reachable network and Recipient opened Receive Authorization. Search ended. Primary Search Again, secondary Back. No cloud invite or persistent listener.

Visible content: phone / No nearby recipients / The search has ended.; Check / Same reachable network / Check / Recipient opened Receive; Search Again; Back

| Action or outcome | Review destination |
|---|---|
| Search Again | [H02](../output/prototypes/homeverb-window-light-v1/screens/H02.png) |
| Back | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### H14 — Codes do not match

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H14.png) · Nested screen

Back Verification Stopped. Restrained red shield outline. Text Codes did not match. No authorization was sent. Primary Start a New Session, secondary Cancel. No Continue Anyway.

Visible content: Codes did not match / No authorization was sent.; Start a New Session; Cancel

| Action or outcome | Review destination |
|---|---|
| Start a New Session | [H02](../output/prototypes/homeverb-window-light-v1/screens/H02.png) |
| Cancel | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### H15 — Session stopped

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H15.png) · Nested screen

Back Sharing Session Ended. Text HomeVerb left the foreground. Nearby discovery and transfer have stopped. Additional short note If sending had started, check the recipient before sending again. Primary Done, secondary View Local Record. No automatic resume.

Visible content: Foreground session stopped / Nearby discovery and transfer have stopped.; If sending had started, check the recipient before sending again.; Done; View Local Record

| Action or outcome | Review destination |
|---|---|
| View Local Record | [S05](../output/prototypes/homeverb-window-light-v1/screens/S05.png) |
| Done | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### H16 — Imported, needs authorization

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/H16.png) · Nested screen

Back Authorization Received. Checklist Transferred (check), Imported securely (check), Authorization validation failed (amber). Text The platform requires a new sign-in. Primary Reauthorize, secondary Remove Imported Authorization. Do not show overall success or fake ready device state.

Visible content: Transferred / Complete / Imported securely / Complete / Authorization check / Failed; A new sign-in is required / Imported authorization is not ready for use.; Reauthorize; Remove Imported Authorization

| Action or outcome | Review destination |
|---|---|
| Reauthorize | [P03](../output/prototypes/homeverb-window-light-v1/screens/P03.png) |
| Remove Imported Authorization | [P11](../output/prototypes/homeverb-window-light-v1/screens/P11.png) |

## Small details that keep it simple

### R05 — Room name validation

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/R05.png) · Nested screen · Sheet over R01

Rooms dimmed under Rename Room sheet. Room name field Living Room with red underline and inline A room with this name already exists. Save disabled, Cancel available. Accessible error text, no generic crash or alert.

Visible content: Room name / Living Room / A room with this name already exists.; Save Room; Cancel

| Action or outcome | Review destination |
|---|---|
| Correct the name | [R02](../output/prototypes/homeverb-window-light-v1/screens/R02.png) |
| Cancel | [R01](../output/prototypes/homeverb-window-light-v1/screens/R01.png) |

### D09 — An empty room

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D09.png) · Root: Devices

HomeVerb device home with Study room chip selected. Quiet desk outline, No devices in Study. Primary Move Existing Devices and secondary Add Device. Bottom nav Devices, Platforms, Settings. Room management remains available.

Visible content: All / Living Room / Bedroom / Study; home / No devices in Study / Make room for the devices you already have.; Move Existing Devices; Add Device

| Action or outcome | Review destination |
|---|---|
| Move Existing Devices | [R03](../output/prototypes/homeverb-window-light-v1/screens/R03.png) |
| Add Device | [A01](../output/prototypes/homeverb-window-light-v1/screens/A01.png) |

### P13 — Platform unavailable

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/P13.png) · Nested screen

Back Example Climate. Unavailable badge. Text This platform has no supported native authorization path yet. Primary Back to Platforms. Secondary Compatibility Details. No browser-secret input, backend setup or misleading Connect button.

Visible content: Unavailable / No supported native authorization path is available yet.; Back to Platforms; Compatibility Details

| Action or outcome | Review destination |
|---|---|
| Back to Platforms | [P02](../output/prototypes/homeverb-window-light-v1/screens/P02.png) |
| Compatibility Details | [S12](../output/prototypes/homeverb-window-light-v1/screens/S12.png) |

### S09 — Clear local data confirmation

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S09.png) · Nested screen · Dialog over S04

Local Data dimmed behind confirmation Remove local data? Checklist explains Rooms, device names, snapshots and local authorizations will be removed. Other installations and platform access are not revoked. Buttons Cancel and Remove Local Data. Destructive action explicit and scoped.

Visible content: Rooms, device names, snapshots and local authorizations will be removed. Other installations and platform access are not revoked.; Remove Local Data; Cancel

| Action or outcome | Review destination |
|---|---|
| Remove local data | [D04](../output/prototypes/homeverb-window-light-v1/screens/D04.png) |
| Cancel | [S04](../output/prototypes/homeverb-window-light-v1/screens/S04.png) |

## Make it feel like yours

### S10 — This device

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S10.png) · Nested screen

Back This Device. Simple phone icon; editable Sharing name field Alex phone. Helper This name is shown only during nearby sharing. Primary Save Name. No registration, real person identity, phone number or cloud profile.

Visible content: Sharing name / Alex phone; Shown only during an explicit nearby sharing session.; Save Name

| Action or outcome | Review destination |
|---|---|
| Save Name | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### S11 — Window Light skin preview

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S11.png) · Nested screen

Preview the currently selected Window Light skin using the same device home. Show Current skin and Included locally. Back returns to Appearance. The only skin remains window_light; previewing does not alter accounts, device state or rooms. This replaces the earlier dark-appearance reference.

Visible content: Window Light Devices preview using the shared tokens; no state mutation.; Current skin / Window Light · Included locally; Soft window light, porcelain surfaces and warm terracotta controls.; Back to Appearance

| Action or outcome | Review destination |
|---|---|
| Back to Appearance | [S02](../output/prototypes/homeverb-window-light-v1/screens/S02.png) |
| Settings | [S01](../output/prototypes/homeverb-window-light-v1/screens/S01.png) |

### S12 — Compatibility

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/S12.png) · Nested screen

Back Compatibility. Explanatory text Support is specific to platform, region and model. Fictional examples Example Home / Planned, Example TV / Pairing research, Example Climate / Authorization blocked. Mock devices / Preview only. Tap a row for evidence details. No fake verified badge or promised whole-brand support.

Visible content: Support is specific to platform, region and model.; Example Home / Planned / Example TV / Pairing research / Example Climate / Authorization blocked / Mock devices / Preview only; Open a row for evidence and limitations. No real platform is verified by these mockups.

| Action or outcome | Review destination |
|---|---|
| Unsupported example | [P13](../output/prototypes/homeverb-window-light-v1/screens/P13.png) |
| Back | [S08](../output/prototypes/homeverb-window-light-v1/screens/S08.png) |

### D10 — Reorder device cards

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D10.png) · Nested screen

Reorder Devices shows the current room and a list of device names, pin state and drag handles. Done saves the complete device permutation after merging the visible reorder with hidden IDs. Room membership and authority stay unchanged. Provide accessible move actions as well as drag.

Visible content: Living Room · Drag handles change the visible order.; Floor lamp / Pinned · ≡ / Living room TV / ≡ / Air sensor / ≡; Room membership stays unchanged. Hidden devices retain their position.; Done

| Action or outcome | Review destination |
|---|---|
| Done | [D02](../output/prototypes/homeverb-window-light-v1/screens/D02.png) |

## The same home, with more space

### W01 — Tablet device home

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/W01.png) · Root: Devices

Wide HomeVerb tablet Devices page. A compact left navigation rail contains exactly Devices selected, Platforms, Settings. Main area header HomeVerb with Search and Add Device. Horizontal room chips All selected, Living Room, Primary Bedroom, Bathroom. Spacious three-column device cards Bedroom AC, Floor lamp, TV, Air sensor, Washer, Fridge. Status and separate power action targets. No fourth share tab, side dashboards or analytics.

Visible content: Three-root rail; responsive device grid or selected-device master/detail view.

| Action or outcome | Review destination |
|---|---|
| Open AC | [W02](../output/prototypes/homeverb-window-light-v1/screens/W02.png) |
| Mobile layout | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

### W02 — Tablet device controls

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/W02.png) · Root: Devices

Wide tablet with identical left three-item navigation rail, Devices selected. Main area split into a narrow local device list (Bedroom AC selected, Floor lamp, TV) and roomy Bedroom AC control panel with 24 C target minus/plus, Room 26 C, Power On, Mode Cool, Fan Auto, Refresh State and information button. Back to Devices in header. This is the same capability control flow adapted to width, not a different feature set.

Visible content: Three-root rail; responsive device grid or selected-device master/detail view.

| Action or outcome | Review destination |
|---|---|
| Back to Devices | [W01](../output/prototypes/homeverb-window-light-v1/screens/W01.png) |

## Clear even when information is missing

### D11 — Cached device state

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D11.png) · Root: Devices

HomeVerb root device page with All selected and exactly three bottom tabs Devices, Platforms, Settings. A subtle banner Saved state / Last read 4 hours ago with Refresh button. Bedroom AC card shows 24 C marked Cached; Floor lamp shows On marked Cached; Air sensor shows Unknown, not zero. Text Connection not checked. Do NOT label devices offline merely because state is old. Search and add remain available.

Visible content: Room filters; device cards; observed state/freshness; explicit refresh; separate quick actions; three-root navigation.

| Action or outcome | Review destination |
|---|---|
| Refresh obtains current data | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |
| Inspect missing sensor data | [C13](../output/prototypes/homeverb-window-light-v1/screens/C13.png) |

### C13 — Unknown sensor reading

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C13.png) · Nested screen

Back Air sensor, Living Room. Read-only screen with Temperature / Not available and Humidity / Not available. Use a dash as placeholder, not 0, 0% or Off. Small Previous read / Yesterday 20:14, Refresh State button. No power switch or writable sensor controls. Brief message No current reading. No bottom navigation.

Visible content: sensor / No current reading / Living Room; Temperature / —  Not available / Humidity / —  Not available / Previous read / Yesterday 20:14; This device is read-only. Missing readings are not zero.; Refresh State

| Action or outcome | Review destination |
|---|---|
| Back to devices | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

### C14 — Invalid control value

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/C14.png) · Nested screen · Sheet over C01

Back Bedroom AC, compact control page with observed target 24 C / Read just now. Bottom sheet Set Temperature with numeric field 23.5 C, red inline error Use whole degrees for this device. Save disabled, Cancel enabled. Helper Supported step: 1 C for this device. Requested invalid input does not replace observed 24 C. No universal temperature range, no bottom navigation.

Visible content: Observed target remains 24°C · Read just now; Target temperature / 23.5°C / Use whole degrees for this device.; Supported step: 1°C for this fictional device.; Save; Cancel

| Action or outcome | Review destination |
|---|---|
| Cancel invalid input | [C01](../output/prototypes/homeverb-window-light-v1/screens/C01.png) |
| Correct input and submit | [C03](../output/prototypes/homeverb-window-light-v1/screens/C03.png) |

### D12 — Larger text layout

[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/D12.png) · Root: Devices

HomeVerb root device page with genuinely larger type and ONE-COLUMN stacked device cards instead of two columns. Room filters horizontally scroll, All selected. Visible cards Bedroom AC / Primary Bedroom / Cooling 24 C and Floor lamp / Living Room / On 65%, with generous power buttons. Freshness line Updated just now. Exactly three bottom nav labels Devices, Platforms, Settings. No clipped or overlapping text. Preserve content hierarchy and accessible touch targets.

Visible content: Room filters; device cards; observed state/freshness; explicit refresh; separate quick actions; three-root navigation.

| Action or outcome | Review destination |
|---|---|
| Standard text layout | [D01](../output/prototypes/homeverb-window-light-v1/screens/D01.png) |

# Appliance card artwork

The owner requested distinct, high-detail refrigerator and dishwasher images for
HomeVerb's device cards. Both assets were generated with the built-in `image_gen`
tool (not the fallback CLI). Each original generation received one targeted
background-extraction edit to reduce exterior white speckles/halos while preserving
the appliance, lighting and transparent alpha. No hand-drawn substitute, scripted
image edit or upscaling was used.

| Asset | Stored dimensions | Format |
|---|---|---|
| [Refrigerator](assets/refrigerator-v1.png) | 1254 x 1254 | RGBA PNG |
| [Dishwasher](assets/dishwasher-v1.png) | 1254 x 1254 | RGBA PNG |

The generation prompts requested 2048-square output; the tool delivered the
1254-square assets listed above. They are generic decorative product renderings,
not a photograph or claim about the exact connected model. Device component and
capability metadata select the artwork; artwork never selects a command or grants
authority. Flutter keeps the source detail and bounds decoding to the card's
physical pixel size. Initial visual acceptance used charcoal cards. The owner
subsequently requested white on-state and slightly gray-white off/unknown-state
cards; the same assets are used on those light surfaces. The original generation
prompts below remain unchanged. Small source-edge matte remnants may remain visible
at full asset resolution.

## Refrigerator generation prompt

```text
Use case: product-mockup
Asset type: high-resolution transparent appliance illustration for a HomeVerb mobile device card, displayed over a charcoal surface.
Primary request: Create one premium photorealistic studio product rendering of a modern freestanding four-door refrigerator, clearly recognizable as a refrigerator at small icon sizes.
Scene/backdrop: genuinely transparent RGBA background, isolated cutout, no backdrop, floor or surrounding room.
Subject: tall full-height multi-door refrigerator with two upper fresh-food doors and two lower freezer doors, all doors closed. Pearl-white enamel fronts, subtle pale brushed-metal edges, clean recessed handles, restrained dark door seams. No external water dispenser. Realistic proportions about twice as tall as wide.
Style/medium: refined high-fidelity product photography / physically based 3D rendering, crisp edge definition and believable fine surface detail. Match a minimalist white wall-air-conditioner catalog product image, not a cartoon or flat pictogram.
Composition/framing: a single entire appliance centered in a square 2048 by 2048 image, near-front view with only a slight right side visible, eye level, product occupies 88 percent of canvas height, generous clear transparent padding, no cropping.
Lighting/mood: soft diffused upper-left studio light, gentle highlights and subtle self-shadow; white surfaces must remain legible on a dark charcoal card.
Constraints: transparent background with clean alpha edges; exactly one refrigerator; no text, numbers, brand, logo, watermark, border, scenery, props or UI. No thick outline or exaggerated perspective.
```

## Dishwasher generation prompt

```text
Use case: product-mockup
Asset type: high-resolution transparent appliance illustration for a HomeVerb mobile device card, displayed over a charcoal surface.
Primary request: Create one premium photorealistic studio rendering of a modern freestanding dishwasher, clearly recognizable as a dishwasher at small icon sizes.
Scene/backdrop: genuinely transparent RGBA background, isolated cutout, no backdrop, floor or surrounding kitchen.
Subject: compact full-height dishwasher with pearl-white enamel front and side panels, a slim brushed-silver top control strip and recessed horizontal handle. Its bottom-hinged front door is partly open about 45 degrees, revealing a tidy dark wire rack with a few white dinner plates and one bowl inside; all dishes belong inside the machine, with no separate props. Realistic dishwasher proportions, roughly 60 cm wide and 85 cm tall. No circular window or washing-machine drum.
Style/medium: refined high-fidelity product photography / physically based 3D rendering, crisp edge definition, believable fine material detail. A coordinated set with a minimalist pearl-white four-door refrigerator and white wall-air-conditioner catalog imagery, not a cartoon or flat pictogram.
Composition/framing: one entire appliance centered in a square 2048 by 2048 image, near-front three-quarter view with only a little right side visible and a slight view down into the rack, appliance occupies about 84 percent of canvas height. Entire door visible, generous transparent padding, no cropping.
Lighting/mood: soft diffused upper-left studio light, gentle highlights and subtle self-shadow, clean white surfaces legible on a dark charcoal card.
Constraints: genuinely transparent background with very clean alpha edges; exactly one dishwasher; no text, numbers, brand, logo, watermark, border, surrounding room or UI. No thick outline, no dramatic reflections, no colored dishes.
```

## Extraction refinement

Each edit targeted only exterior white speckles, patches and halos, preserving the
complete appliance silhouette, material detail, viewing angle, pose and genuine
transparency. The selected edited versions are the versioned workspace assets
linked above. No live account material or device identifiers were supplied to the
image tool.

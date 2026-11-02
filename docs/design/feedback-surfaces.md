# Feedback surfaces

## Visual contract

Feedback is not a selectable setting. Use `FeedbackToast` through the
page-scoped `FeedbackQueue` for explicit, short-lived foreground results.
Its dark floating surface, status icon, shadow and divided action area remain
visually distinct from white device and option cards in both skins. Only the
labeled action is clickable; tapping the message must do nothing.

Use `FeedbackNotice` for a condition that remains actionable after the moment
has passed: expired authorization, sign-in correction, uncertain credential
persistence, or an operation that needs inspection. The pale notice has a slim
leading color edge, no selected state, no whole-row gesture, and at most one
local action. Its outlined action moves below the message for enlarged text.
Never replace a destructive or elevated-operation confirmation with a Toast.

## Semantics and lifetime

- Successful explicit state reads may say readings updated; completed account
  retrieval may say device list updated. Neither means a write was verified.
- Partial retrieval is partial, not success. Saved cards/readings remain intact.
- Expired authorization has one persistent sign-in notice. Do not stack raw
  codes, duplicate warnings, reauthorize and refresh buttons in the same area.
- Silent automatic reads remain silent. They never enqueue feedback.
- Safe retry is offered only for explicit read operations, never uncertain
  writes or credential recovery. Unknown writes keep their receipt/details path.
- Call `FeedbackQueue.clear` on navigation, editor changes and backgrounding.
  Capture navigation/activity/account/receipt identity before asynchronous work
  and validate it before publishing or running a feedback action.
- Queue at most three distinct keys, suppress duplicate keys, and drop stale
  entries. Clearing increments a generation so old close callbacks cannot
  remove a newer entry with the same key.
- Announce through a live region. Normal feedback lasts eight seconds; screen
  readers get thirty seconds and actionable notices stay until dismissed.
  Reduced-motion mode disables Toast transitions. Persistent notices do not
  animate. Use 48 logical-pixel action targets and support 320-pixel layouts,
  doubled text, high contrast and both skins.
- Consumer messages use authored catalog text and allowlisted error meanings.
  Never render raw provider response text or unknown provider-controlled codes.
  Explicit diagnostic/receipt views retain their separate diagnostic contract.

## Extending to additional platforms

Reuse these presentation components for future platform adapters. Select
feedback by outcome, uncertainty and remediation, not vendor-name conditionals.
The presentation layer must not infer authorization from saved credentials,
retry a write, invent synchronization features, or change Core authority.

The approved preview illustrated visual placement with fictional data. Its
illustrative automatic-sync switch is not a new product capability and is not
implemented by this change.

///
/// Generated file. Do not edit.
///
// coverage:ignore-file
// ignore_for_file: type=lint, unused_import
// dart format off

part of 'strings.g.dart';

// Path: <root>
typedef TranslationsEnUs = Translations; // ignore: unused_element
class Translations with BaseTranslations<AppLocale, Translations> {
	/// Returns the current translations of the given [context].
	///
	/// Usage:
	/// final t = Translations.of(context);
	static Translations of(BuildContext context) => InheritedLocaleData.of<AppLocale, Translations>(context).translations;

	/// You can call this constructor and build your own translation instance of this locale.
	/// Constructing via the enum [AppLocale.build] is preferred.
	Translations({Map<String, Node>? overrides, PluralResolver? cardinalResolver, PluralResolver? ordinalResolver, TranslationMetadata<AppLocale, Translations>? meta})
		: assert(overrides == null, 'Set "translation_overrides: true" in order to enable this feature.'),
		  $meta = meta ?? TranslationMetadata(
		    locale: AppLocale.enUs,
		    overrides: overrides ?? {},
		    cardinalResolver: cardinalResolver,
		    ordinalResolver: ordinalResolver,
		  );

	/// Metadata for the translations of <en-US>.
	@override final TranslationMetadata<AppLocale, Translations> $meta;

	late final Translations _root = this; // ignore: unused_field

	Translations $copyWith({TranslationMetadata<AppLocale, Translations>? meta}) => Translations(meta: meta ?? this.$meta);

	// Translations

	/// en-US: 'HomeVerb'
	String get appName => 'HomeVerb';

	/// en-US: 'Fictional device. No real appliance is connected.'
	String get appMockNotice => 'Fictional device. No real appliance is connected.';

	/// en-US: 'Devices'
	String get deviceList => 'Devices';

	/// en-US: 'Discover devices'
	String get deviceDiscover => 'Discover devices';

	/// en-US: 'No devices yet. Start discovery to add a device.'
	String get deviceEmpty => 'No devices yet. Start discovery to add a device.';

	/// en-US: 'Refresh state'
	String get deviceRefresh => 'Refresh state';

	/// en-US: 'Capabilities'
	String get deviceCapabilities => 'Capabilities';

	/// en-US: 'Unknown'
	String get deviceUnknown => 'Unknown';

	/// en-US: 'Cached observation'
	String get stateCached => 'Cached observation';

	/// en-US: 'Live observation'
	String get stateLive => 'Live observation';

	/// en-US: 'This observation is stale.'
	String get stateStale => 'This observation is stale.';

	/// en-US: 'Observed at {time}'
	String stateObservedAt({required Object time}) => 'Observed at ${time}';

	/// en-US: 'Received at {time}'
	String stateReceivedAt({required Object time}) => 'Received at ${time}';

	/// en-US: 'Requested: {value}'
	String statePendingTarget({required Object value}) => 'Requested: ${value}';

	/// en-US: 'On'
	String get powerOn => 'On';

	/// en-US: 'Off'
	String get powerOff => 'Off';

	/// en-US: 'Target temperature'
	String get climateTargetTemperature => 'Target temperature';

	/// en-US: 'Mode'
	String get climateMode => 'Mode';

	/// en-US: 'Apply'
	String get actionApply => 'Apply';

	/// en-US: 'Cancel'
	String get actionCancel => 'Cancel';

	/// en-US: 'Accepted; not yet confirmed'
	String get executionAccepted => 'Accepted; not yet confirmed';

	/// en-US: 'Verified by fresh evidence'
	String get executionVerified => 'Verified by fresh evidence';

	/// en-US: 'Failed'
	String get executionFailed => 'Failed';

	/// en-US: 'May have executed; refresh to verify'
	String get executionUnknown => 'May have executed; refresh to verify';

	/// en-US: 'Results differ; inspect each operation'
	String get executionPartial => 'Results differ; inspect each operation';

	/// en-US: 'Matching state does not prove that this request caused it.'
	String get executionNoCausality => 'Matching state does not prove that this request caused it.';

	/// en-US: 'Check the values against this device's capabilities.'
	String get errorINVALIDARGUMENT => 'Check the values against this device\'s capabilities.';

	/// en-US: 'This operation needs permission from a trusted user interface.'
	String get errorPOLICYDENIED => 'This operation needs permission from a trusted user interface.';

	/// en-US: 'Return to the foreground to start an operation.'
	String get errorHOSTBACKGROUND => 'Return to the foreground to start an operation.';

	/// en-US: 'Another operation is using this resource.'
	String get errorRESOURCEBUSY => 'Another operation is using this resource.';

	/// en-US: 'Refresh the state before considering another write.'
	String get errorWRITEOUTCOMEUNKNOWN => 'Refresh the state before considering another write.';

	/// en-US: 'The operation could not finish. Error code: {code}'
	String errorGeneric({required Object code}) => 'The operation could not finish. Error code: ${code}';

	/// en-US: 'Language'
	String get settingsLanguage => 'Language';

	/// en-US: 'True'
	String get booleanTrue => 'True';

	/// en-US: 'False'
	String get booleanFalse => 'False';

	/// en-US: 'Cool'
	String get climateModeCool => 'Cool';

	/// en-US: 'Heat'
	String get climateModeHeat => 'Heat';

	/// en-US: 'Fan'
	String get climateModeFan => 'Fan';

	/// en-US: 'Dry'
	String get climateModeDry => 'Dry';

	/// en-US: 'Auto'
	String get climateModeAuto => 'Auto';

	/// en-US: 'All'
	String get roomAll => 'All';

	/// en-US: 'Unassigned'
	String get roomUnassigned => 'Unassigned';

	/// en-US: 'Manage rooms'
	String get roomManage => 'Manage rooms';

	/// en-US: 'Create room'
	String get roomCreate => 'Create room';

	/// en-US: 'Room name'
	String get roomName => 'Room name';

	/// en-US: 'Rename room'
	String get roomRename => 'Rename room';

	/// en-US: 'Delete room'
	String get roomDelete => 'Delete room';

	/// en-US: 'Devices will move to Unassigned. Device access will remain unchanged.'
	String get roomDeleteNotice => 'Devices will move to Unassigned. Device access will remain unchanged.';

	/// en-US: 'Device name'
	String get deviceName => 'Device name';

	/// en-US: 'Favorite'
	String get deviceFavorite => 'Favorite';

	/// en-US: 'Device order'
	String get deviceOrder => 'Device order';

	/// en-US: 'This room is no longer available. Refresh the room list.'
	String get errorROOMNOTFOUND => 'This room is no longer available. Refresh the room list.';

	/// en-US: 'A room already uses this name.'
	String get errorROOMNAMECONFLICT => 'A room already uses this name.';

	/// en-US: 'This request was already recorded. Its receipt expired; refresh state before considering another write.'
	String get errorREQUESTRETIRED => 'This request was already recorded. Its receipt expired; refresh state before considering another write.';

	/// en-US: 'Move selected devices'
	String get roomMoveSelected => 'Move selected devices';

	/// en-US: 'Selected devices: {count}'
	String roomSelectionCount({required Object count}) => 'Selected devices: ${count}';

	/// en-US: 'Room assignment saved.'
	String get roomAssignmentSaved => 'Room assignment saved.';

	/// en-US: 'Recent operations'
	String get executionHistory => 'Recent operations';

	/// en-US: 'Operations for this device'
	String get executionDeviceHistory => 'Operations for this device';

	/// en-US: 'No retained operations for this device.'
	String get executionHistoryEmpty => 'No retained operations for this device.';

	/// en-US: 'Secure credential storage is unavailable. Configure the system credential store.'
	String get errorSECRETSTOREUNAVAILABLE => 'Secure credential storage is unavailable. Configure the system credential store.';

	/// en-US: 'Unlock the system credential store before trying again.'
	String get errorSECRETSTORELOCKED => 'Unlock the system credential store before trying again.';

	/// en-US: 'Multiple entries match this credential reference. Resolve the conflict before continuing.'
	String get errorSECRETSTORECONFLICT => 'Multiple entries match this credential reference. Resolve the conflict before continuing.';

	/// en-US: 'The credential change may have completed. Check its status before trying again.'
	String get errorSECRETSTOREWRITEUNKNOWN => 'The credential change may have completed. Check its status before trying again.';

	/// en-US: 'Diagnostics'
	String get diagnosticsTitle => 'Diagnostics';

	/// en-US: 'Preview report'
	String get diagnosticsPreview => 'Preview report';

	/// en-US: 'Save report'
	String get diagnosticsExport => 'Save report';

	/// en-US: 'Local counts and recent outcome summaries. No live connection checks.'
	String get diagnosticsScopeNotice => 'Local counts and recent outcome summaries. No live connection checks.';

	/// en-US: 'Identifiers, names, state values, arguments, and raw messages are omitted. Review the report before sharing.'
	String get diagnosticsPrivacyNotice => 'Identifiers, names, state values, arguments, and raw messages are omitted. Review the report before sharing.';

	/// en-US: 'Includes at most {count} stored receipts. Composite operations may appear more than once.'
	String diagnosticsHistoryNotice({required Object count}) => 'Includes at most ${count} stored receipts. Composite operations may appear more than once.';

	/// en-US: 'The state data is inconsistent. Refresh to obtain a valid observation.'
	String get errorINVALIDOBSERVATION => 'The state data is inconsistent. Refresh to obtain a valid observation.';

	/// en-US: 'The discovered connection conflicts with this saved device. Its existing identity has been preserved.'
	String get errorDEVICEIDENTITYCONFLICT => 'The discovered connection conflicts with this saved device. Its existing identity has been preserved.';

	/// en-US: 'This connection cannot be linked with the current adapter or storage.'
	String get errorCONNECTIONLINKUNSUPPORTED => 'This connection cannot be linked with the current adapter or storage.';

	/// en-US: 'The device or connection preview changed. Review it again before confirming.'
	String get errorCONNECTIONLINKINVALID => 'The device or connection preview changed. Review it again before confirming.';

	/// en-US: 'The connection preview expired. Review it again before confirming.'
	String get errorCONNECTIONLINKEXPIRED => 'The connection preview expired. Review it again before confirming.';

	/// en-US: 'The device or network scan preview changed. Review it again before confirming.'
	String get errorLANSCANINVALID => 'The device or network scan preview changed. Review it again before confirming.';

	/// en-US: 'The network scan preview expired. Review it again before confirming.'
	String get errorLANSCANEXPIRED => 'The network scan preview expired. Review it again before confirming.';

	/// en-US: 'No receipt was found in this profile. This does not establish whether the device acted; do not automatically resend.'
	String get errorREQUESTNOTFOUND => 'No receipt was found in this profile. This does not establish whether the device acted; do not automatically resend.';

	/// en-US: 'Reconnect this platform account to continue.'
	String get errorAUTHREQUIRED => 'Reconnect this platform account to continue.';

	/// en-US: 'This local account is no longer available. Reload the account list.'
	String get errorACCOUNTNOTFOUND => 'This local account is no longer available. Reload the account list.';

	/// en-US: 'The returned identity does not match the saved account binding. Your saved account and devices are unchanged. If you used the same account, this identity mismatch needs investigation.'
	String get errorACCOUNTIDENTITYCONFLICT => 'The returned identity does not match the saved account binding. Your saved account and devices are unchanged. If you used the same account, this identity mismatch needs investigation.';

	/// en-US: 'Saved authorization needs recovery. Open this platform account to recover it.'
	String get errorACCOUNTSESSIONUNCONFIRMED => 'Saved authorization needs recovery. Open this platform account to recover it.';

	/// en-US: 'The platform returned an unsupported response.'
	String get errorMIDEARESPONSEINVALID => 'The platform returned an unsupported response.';

	/// en-US: 'The platform rejected the request.'
	String get errorMIDEAREQUESTREJECTED => 'The platform rejected the request.';

	/// en-US: 'The platform data could not be retrieved.'
	String get errorMIDEAREQUESTFAILED => 'The platform data could not be retrieved.';

	/// en-US: 'Some platform devices could not be retrieved. Existing device cards were kept.'
	String get errorMIDEAINVENTORYPARTIAL => 'Some platform devices could not be retrieved. Existing device cards were kept.';

	/// en-US: 'Account connection is not available in this host.'
	String get errorAUTHFLOWUNSUPPORTED => 'Account connection is not available in this host.';

	/// en-US: 'This account or password format is not supported by the current connection method.'
	String get errorMIDEALOGININPUTUNSUPPORTED => 'This account or password format is not supported by the current connection method.';

	/// en-US: 'Authorization has not been checked.'
	String get accountAuthorizationNotChecked => 'Authorization has not been checked.';

	/// en-US: 'Connecting and retrieving devices...'
	String get accountConnecting => 'Connecting and retrieving devices...';

	/// en-US: 'Account connection succeeded, but devices could not be loaded.'
	String get accountConnectedRetrievalFailed => 'Account connection succeeded, but devices could not be loaded.';

	/// en-US: 'Complete local cleanup before connecting this platform account again.'
	String get errorACCOUNTREMOVALPENDING => 'Complete local cleanup before connecting this platform account again.';

	/// en-US: 'The linked devices changed. Review the removal scope again.'
	String get errorACCOUNTSCOPECHANGED => 'The linked devices changed. Review the removal scope again.';

	/// en-US: 'A linked device has other connections. This account cannot be removed safely yet.'
	String get errorACCOUNTREMOVALUNSUPPORTED => 'A linked device has other connections. This account cannot be removed safely yet.';

	/// en-US: 'Remove this account and {count} linked device cards from this installation? Rooms and operation receipts will remain. Other installations and the platform account are unaffected.'
	String accountRemovalConfirm({required Object count}) => 'Remove this account and ${count} linked device cards from this installation? Rooms and operation receipts will remain. Other installations and the platform account are unaffected.';

	/// en-US: 'Local cleanup is incomplete. This local account is disabled.'
	String get accountRemovalPending => 'Local cleanup is incomplete. This local account is disabled.';

	/// en-US: 'The account was removed from this installation. Platform authorization elsewhere was not revoked.'
	String get accountRemovalRemoved => 'The account was removed from this installation. Platform authorization elsewhere was not revoked.';

	/// en-US: 'Retry local cleanup'
	String get accountRemovalRetry => 'Retry local cleanup';

	/// en-US: 'Clear cached device state from this installation? Accounts, rooms and operation receipts will remain. Active reads may save new snapshots.'
	String get cleanupCacheConfirm => 'Clear cached device state from this installation? Accounts, rooms and operation receipts will remain. Active reads may save new snapshots.';

	/// en-US: 'Cleared {count} cached snapshots.'
	String cleanupCacheDone({required Object count}) => 'Cleared ${count} cached snapshots.';

	/// en-US: 'Clear detailed operation history, including uncertain outcomes? Request IDs will remain reserved to prevent replay. Devices and cached state will remain.'
	String get cleanupHistoryConfirm => 'Clear detailed operation history, including uncertain outcomes? Request IDs will remain reserved to prevent replay. Devices and cached state will remain.';

	/// en-US: 'Cleared {count} operation receipts.'
	String cleanupHistoryDone({required Object count}) => 'Cleared ${count} operation receipts.';

	/// en-US: 'An operation is still being recorded. Wait for it to finish, then retry clearing history.'
	String get cleanupHistoryBusy => 'An operation is still being recorded. Wait for it to finish, then retry clearing history.';

	/// en-US: 'Recover saved authorization'
	String get accountRecoveryAction => 'Recover saved authorization';

	/// en-US: 'Use this account\u2019s saved authorization to recover access and reload devices? Existing rooms and device names will be kept.'
	String get accountRecoveryConfirm => 'Use this account\u2019s saved authorization to recover access and reload devices? Existing rooms and device names will be kept.';

	/// en-US: 'Saved authorization recovered and devices loaded.'
	String get accountRecoveryComplete => 'Saved authorization recovered and devices loaded.';

	/// en-US: 'Recovery is incomplete. Existing local account data has been kept.'
	String get accountRecoveryPending => 'Recovery is incomplete. Existing local account data has been kept.';

	/// en-US: 'The account is stored, but devices could not be loaded. Review the retrieval error.'
	String get accountRecoveryRetrievalFailed => 'The account is stored, but devices could not be loaded. Review the retrieval error.';

	/// en-US: 'Saved account data is missing or damaged. Recovery and sign-in cannot safely restore its device bindings. Removing and reconnecting is a separate action that loses its saved organization.'
	String get accountRecoveryUnrecoverable => 'Saved account data is missing or damaged. Recovery and sign-in cannot safely restore its device bindings. Removing and reconnecting is a separate action that loses its saved organization.';

	/// en-US: 'Saved authorization has expired. Sign in to the original account to retain existing device organization when its saved account data is intact.'
	String get accountRecoveryExpired => 'Saved authorization has expired. Sign in to the original account to retain existing device organization when its saved account data is intact.';

	/// en-US: 'Sign in to the original account'
	String get accountReauthorizationAction => 'Sign in to the original account';

	/// en-US: 'Sign in again to this platform account? Existing rooms and device names will be kept only if the saved account identity matches. Device control permissions will stay the same.'
	String get accountReauthorizationConfirm => 'Sign in again to this platform account? Existing rooms and device names will be kept only if the saved account identity matches. Device control permissions will stay the same.';

	/// en-US: 'Fan speed'
	String get vendorMideaAcWindSpeed => 'Fan speed';

	/// en-US: 'Set fan speed'
	String get vendorMideaAcSetFanSpeed => 'Set fan speed';

	/// en-US: 'Silent'
	String get vendorMideaAcWindSpeed20 => 'Silent';

	/// en-US: 'Low'
	String get vendorMideaAcWindSpeed40 => 'Low';

	/// en-US: 'Medium'
	String get vendorMideaAcWindSpeed60 => 'Medium';

	/// en-US: 'High'
	String get vendorMideaAcWindSpeed80 => 'High';

	/// en-US: 'Full'
	String get vendorMideaAcWindSpeed100 => 'Full';

	/// en-US: 'Auto'
	String get vendorMideaAcWindSpeed102 => 'Auto';

	/// en-US: 'This device was missing from the last complete platform list. Refresh the account before trying again.'
	String get errorMIDEADEVICENOTLISTED => 'This device was missing from the last complete platform list. Refresh the account before trying again.';

	/// en-US: 'Refresh this platform account to confirm its device list before reading or controlling devices.'
	String get errorMIDEAINVENTORYREQUIRED => 'Refresh this platform account to confirm its device list before reading or controlling devices.';

	/// en-US: 'Inspect saved device list'
	String get accountInventoryInspect => 'Inspect saved device list';

	/// en-US: 'Present in last complete list'
	String get inventoryMembershipListed => 'Present in last complete list';

	/// en-US: 'Missing from last complete list'
	String get inventoryMembershipNotListed => 'Missing from last complete list';

	/// en-US: 'Device list needs confirmation'
	String get inventoryMembershipUnconfirmed => 'Device list needs confirmation';

	/// en-US: 'Mode'
	String get vendorMideaAcMode => 'Mode';

	/// en-US: 'Cool'
	String get vendorMideaAcModeCool => 'Cool';

	/// en-US: 'Heat'
	String get vendorMideaAcModeHeat => 'Heat';

	/// en-US: 'Auto'
	String get vendorMideaAcModeAuto => 'Auto';

	/// en-US: 'Dry'
	String get vendorMideaAcModeDry => 'Dry';

	/// en-US: 'Fan'
	String get vendorMideaAcModeFan => 'Fan';

	/// en-US: 'Turn on and set mode'
	String get vendorMideaAcActivateMode => 'Turn on and set mode';

	/// en-US: 'Turn on {device} and switch to {mode}?'
	String vendorMideaAcActivateModeConfirm({required Object device, required Object mode}) => 'Turn on ${device} and switch to ${mode}?';

	/// en-US: 'The confirmation expired. Review the command again before confirming.'
	String get errorAPPROVALEXPIRED => 'The confirmation expired. Review the command again before confirming.';

	/// en-US: 'The device or confirmation context changed. Review the command again.'
	String get errorAPPROVALINVALID => 'The device or confirmation context changed. Review the command again.';

	/// en-US: 'Start wash cycle'
	String get vendorMideaDishwasherStartCycle => 'Start wash cycle';

	/// en-US: 'Pause wash cycle'
	String get vendorMideaDishwasherPauseCycle => 'Pause wash cycle';

	/// en-US: 'Resume wash cycle'
	String get vendorMideaDishwasherResumeCycle => 'Resume wash cycle';

	/// en-US: 'Cancel wash cycle'
	String get vendorMideaDishwasherCancelCycle => 'Cancel wash cycle';

	/// en-US: 'Request a pause on {device}? This does not confirm that the door is safe to open.'
	String vendorMideaDishwasherPauseCycleConfirm({required Object device}) => 'Request a pause on ${device}? This does not confirm that the door is safe to open.';

	/// en-US: 'Request a pause on {device} and set {additional}? This does not confirm that the door is safe to open.'
	String vendorMideaDishwasherPauseCycleConfirmAdditional({required Object device, required Object additional}) => 'Request a pause on ${device} and set ${additional}? This does not confirm that the door is safe to open.';

	/// en-US: 'Request that {device} resume its cycle? This can consume water and heat.'
	String vendorMideaDishwasherResumeCycleConfirm({required Object device}) => 'Request that ${device} resume its cycle? This can consume water and heat.';

	/// en-US: 'Request that {device} resume with {additional}? This sets the additional option and can consume water and heat.'
	String vendorMideaDishwasherResumeCycleConfirmAdditional({required Object device, required Object additional}) => 'Request that ${device} resume with ${additional}? This sets the additional option and can consume water and heat.';

	/// en-US: 'Request cancellation on {device}? This may terminate the program; draining and rollback are not confirmed.'
	String vendorMideaDishwasherCancelCycleConfirm({required Object device}) => 'Request cancellation on ${device}? This may terminate the program; draining and rollback are not confirmed.';

	/// en-US: 'Request cancellation on {device} and set {additional}? This may terminate the program and does not confirm draining or rollback.'
	String vendorMideaDishwasherCancelCycleConfirmAdditional({required Object device, required Object additional}) => 'Request cancellation on ${device} and set ${additional}? This may terminate the program and does not confirm draining or rollback.';

	/// en-US: 'Power on'
	String get vendorMideaDishwasherPowerOn => 'Power on';

	/// en-US: 'Power off'
	String get vendorMideaDishwasherPowerOff => 'Power off';

	/// en-US: 'Power on {device}? No wash program is selected by this request.'
	String vendorMideaDishwasherPowerOnConfirm({required Object device}) => 'Power on ${device}? No wash program is selected by this request.';

	/// en-US: 'Power on {device} with {additional}? This sets the additional option without selecting a wash program.'
	String vendorMideaDishwasherPowerOnConfirmAdditional({required Object device, required Object additional}) => 'Power on ${device} with ${additional}? This sets the additional option without selecting a wash program.';

	/// en-US: 'Power off {device}? This may interrupt a program and does not confirm draining or safe door opening.'
	String vendorMideaDishwasherPowerOffConfirm({required Object device}) => 'Power off ${device}? This may interrupt a program and does not confirm draining or safe door opening.';

	/// en-US: 'Power off {device} and set {additional}? This may interrupt a program and does not confirm draining or safe door opening.'
	String vendorMideaDishwasherPowerOffConfirmAdditional({required Object device, required Object additional}) => 'Power off ${device} and set ${additional}? This may interrupt a program and does not confirm draining or safe door opening.';

	/// en-US: 'Reported cycle operator'
	String get vendorMideaDishwasherOperator => 'Reported cycle operator';

	/// en-US: 'Start a wash on {device} using {program}? This can consume water and heat.'
	String vendorMideaDishwasherStartCycleConfirm({required Object device, required Object program}) => 'Start a wash on ${device} using ${program}? This can consume water and heat.';

	/// en-US: 'Start a wash on {device} using {program} with {additional}? This sets the additional option and can consume water and heat.'
	String vendorMideaDishwasherStartCycleConfirmAdditional({required Object device, required Object program, required Object additional}) => 'Start a wash on ${device} using ${program} with ${additional}? This sets the additional option and can consume water and heat.';

	/// en-US: 'Wash program'
	String get vendorMideaDishwasherProgram => 'Wash program';

	/// en-US: 'Additional option'
	String get vendorMideaDishwasherAdditional => 'Additional option';

	/// en-US: 'Auto wash'
	String get vendorMideaDishwasherProgramAutoWash => 'Auto wash';

	/// en-US: 'Intensive wash'
	String get vendorMideaDishwasherProgramStrongWash => 'Intensive wash';

	/// en-US: 'Standard wash'
	String get vendorMideaDishwasherProgramStandardWash => 'Standard wash';

	/// en-US: 'Eco wash'
	String get vendorMideaDishwasherProgramEcoWash => 'Eco wash';

	/// en-US: 'Glass wash'
	String get vendorMideaDishwasherProgramGlassWash => 'Glass wash';

	/// en-US: '90-minute wash'
	String get vendorMideaDishwasherProgram90minWash => '90-minute wash';

	/// en-US: 'Quick wash'
	String get vendorMideaDishwasherProgramFastWash => 'Quick wash';

	/// en-US: 'Soak wash'
	String get vendorMideaDishwasherProgramSoakWash => 'Soak wash';

	/// en-US: 'One-hour wash'
	String get vendorMideaDishwasherProgramHourWash => 'One-hour wash';

	/// en-US: 'Quiet night wash'
	String get vendorMideaDishwasherProgramQuietnightWash => 'Quiet night wash';

	/// en-US: 'Hygiene wash'
	String get vendorMideaDishwasherProgramGerm => 'Hygiene wash';

	/// en-US: 'Self-clean'
	String get vendorMideaDishwasherProgramSelfClean => 'Self-clean';

	/// en-US: 'Fruit wash'
	String get vendorMideaDishwasherProgramFruitWash => 'Fruit wash';

	/// en-US: 'Gentle wash'
	String get vendorMideaDishwasherProgramSoftWash => 'Gentle wash';

	/// en-US: 'Disinfect only'
	String get vendorMideaDishwasherProgramSingleDisinfect => 'Disinfect only';

	/// en-US: 'Seafood wash'
	String get vendorMideaDishwasherProgramSeafoodWash => 'Seafood wash';

	/// en-US: 'Hotpot wash'
	String get vendorMideaDishwasherProgramHotpotWash => 'Hotpot wash';

	/// en-US: 'No additional option'
	String get vendorMideaDishwasherAdditional0 => 'No additional option';

	/// en-US: 'One extra rinse'
	String get vendorMideaDishwasherAdditional9 => 'One extra rinse';

	/// en-US: 'Two extra rinses'
	String get vendorMideaDishwasherAdditional10 => 'Two extra rinses';

	/// en-US: 'Few dishes with one extra rinse'
	String get vendorMideaDishwasherAdditional13 => 'Few dishes with one extra rinse';

	/// en-US: 'Few dishes with two extra rinses'
	String get vendorMideaDishwasherAdditional14 => 'Few dishes with two extra rinses';

	/// en-US: 'Review control permissions'
	String get controlOptions => 'Review control permissions';

	/// en-US: 'These options describe supported commands. Permission has not been checked and no control is enabled by this review.'
	String get controlOptionsNotice => 'These options describe supported commands. Permission has not been checked and no control is enabled by this review.';

	/// en-US: 'No reviewed control options are available for this device.'
	String get controlOptionsEmpty => 'No reviewed control options are available for this device.';

	/// en-US: 'Allow selected controls'
	String get controlOptionsAllow => 'Allow selected controls';

	/// en-US: 'Allowed controls for this session'
	String get controlPermissions => 'Allowed controls for this session';

	/// en-US: 'No controls are allowed for this device on this connection.'
	String get controlPermissionsEmpty => 'No controls are allowed for this device on this connection.';

	/// en-US: 'These permissions apply to this session. Device availability and any confirmation needed are checked when you use a control.'
	String get controlPermissionsNotice => 'These permissions apply to this session. Device availability and any confirmation needed are checked when you use a control.';

	/// en-US: 'Find this device on the local network'
	String get lanScanTitle => 'Find this device on the local network';

	/// en-US: 'Scan selected network'
	String get lanScanConfirm => 'Scan selected network';

	/// en-US: 'Scan only the selected network while this app is in the foreground. Finding a candidate does not add a connection or verify the device.'
	String get lanScanNotice => 'Scan only the selected network while this app is in the foreground. Finding a candidate does not add a connection or verify the device.';

	/// en-US: 'Selected network: {address}/{prefix}'
	String lanScanNetwork({required Object address, required Object prefix}) => 'Selected network: ${address}/${prefix}';

	/// en-US: 'No matching candidate was observed.'
	String get lanScanNotObserved => 'No matching candidate was observed.';

	/// en-US: 'The scan was incomplete. No candidate was selected.'
	String get lanScanIncomplete => 'The scan was incomplete. No candidate was selected.';

	/// en-US: 'Conflicting candidates were found. No candidate was selected.'
	String get lanScanAmbiguous => 'Conflicting candidates were found. No candidate was selected.';

	/// en-US: 'One candidate was found. Device verification and connection setup are still required.'
	String get lanScanUniqueCandidate => 'One candidate was found. Device verification and connection setup are still required.';

	/// en-US: 'Allow network-state access before selecting a local network.'
	String get errorLANNETWORKPERMISSIONREQUIRED => 'Allow network-state access before selecting a local network.';

	/// en-US: 'Android network information is unavailable. Try again in the foreground.'
	String get errorLANNETWORKUNAVAILABLE => 'Android network information is unavailable. Try again in the foreground.';

	/// en-US: 'The selected network changed or expired. Select it again and review a new scan preview.'
	String get errorLANNETWORKCHANGED => 'The selected network changed or expired. Select it again and review a new scan preview.';

	/// en-US: 'Local scanning is unavailable with the current VPN route. Cloud operations remain separate.'
	String get errorLANNETWORKUNSUPPORTED => 'Local scanning is unavailable with the current VPN route. Cloud operations remain separate.';

	/// en-US: 'Last complete device list: {time}'
	String accountInventoryLastComplete({required Object time}) => 'Last complete device list: ${time}';

	/// en-US: 'No complete device-list time recorded'
	String get accountInventoryNeverRecorded => 'No complete device-list time recorded';

	/// en-US: 'Recorded using this device\u2019s clock. This does not confirm current authorization or appliance status.'
	String get accountInventoryTimeHint => 'Recorded using this device\u2019s clock. This does not confirm current authorization or appliance status.';

	/// en-US: 'Add a connection'
	String get connectionLinkTitle => 'Add a connection';

	/// en-US: 'Review the existing device and the proposed connection. This keeps the same device and room assignments.'
	String get connectionLinkScope => 'Review the existing device and the proposed connection. This keeps the same device and room assignments.';

	/// en-US: 'The new route can affect automatic reads and enable existing permissions for that transport. It does not grant new permission.'
	String get connectionLinkGrantsHint => 'The new route can affect automatic reads and enable existing permissions for that transport. It does not grant new permission.';

	/// en-US: 'Add this connection'
	String get connectionLinkConfirm => 'Add this connection';

	/// en-US: 'The connection change was interrupted. Check saved connections before trying again.'
	String get connectionLinkInterrupted => 'The connection change was interrupted. Check saved connections before trying again.';

	/// en-US: 'Inspect duplicate device records'
	String get accountInspectDeviceConflicts => 'Inspect duplicate device records';

	/// en-US: 'These saved records may refer to the same appliance. Inspection does not confirm physical identity or merge devices.'
	String get accountDeviceConflictsNotice => 'These saved records may refer to the same appliance. Inspection does not confirm physical identity or merge devices.';

	/// en-US: 'Conflict inspection supports at most 64 mainland Meiju accounts.'
	String get errorMIDEACONFLICTLIMIT => 'Conflict inspection supports at most 64 mainland Meiju accounts.';

	/// en-US: 'Start fictional cycle'
	String get vendorExampleCycleStart => 'Start fictional cycle';

	/// en-US: 'Start one fictional cycle on {device} using {program}? This development action does not operate a physical appliance.'
	String vendorExampleCycleStartConfirm({required Object device, required Object program}) => 'Start one fictional cycle on ${device} using ${program}? This development action does not operate a physical appliance.';

	/// en-US: 'normal'
	String get vendorExampleCycleProgramNormal => 'normal';

	/// en-US: 'System default'
	String get mobileSystemDefault => 'System default';

	/// en-US: 'Simplified Chinese'
	String get mobileSimplifiedChinese => 'Simplified Chinese';

	/// en-US: 'English (United States)'
	String get mobileEnglishUnitedStates => 'English (United States)';

	/// en-US: 'Settings'
	String get mobileSettings => 'Settings';

	/// en-US: 'Platforms'
	String get mobilePlatforms => 'Platforms';

	/// en-US: 'Appearance'
	String get mobileAppearance => 'Appearance';

	/// en-US: 'Add Device'
	String get mobileAddDevice => 'Add Device';

	/// en-US: 'Add device'
	String get mobileAddDevice419d18 => 'Add device';

	/// en-US: 'Add Platform'
	String get mobileAddPlatform => 'Add Platform';

	/// en-US: 'Add platform'
	String get mobileAddPlatformE4d683 => 'Add platform';

	/// en-US: 'Add Room'
	String get mobileAddRoom => 'Add Room';

	/// en-US: 'Add More'
	String get mobileAddMore => 'Add More';

	/// en-US: 'Add Another Platform'
	String get mobileAddAnotherPlatform => 'Add Another Platform';

	/// en-US: 'About'
	String get mobileAbout => 'About';

	/// en-US: 'Back'
	String get mobileBack => 'Back';

	/// en-US: 'Close'
	String get mobileClose => 'Close';

	/// en-US: 'Continue'
	String get mobileContinue => 'Continue';

	/// en-US: 'Save'
	String get mobileSave => 'Save';

	/// en-US: 'Name'
	String get mobileName => 'Name';

	/// en-US: 'Room'
	String get mobileRoom => 'Room';

	/// en-US: 'Model'
	String get mobileModel => 'Model';

	/// en-US: 'Connection'
	String get mobileConnection => 'Connection';

	/// en-US: 'Connections'
	String get mobileConnections => 'Connections';

	/// en-US: 'Cloud'
	String get mobileCloud => 'Cloud';

	/// en-US: 'Local network'
	String get mobileLocalNetwork => 'Local network';

	/// en-US: 'Provider cloud'
	String get mobileProviderCloud => 'Provider cloud';

	/// en-US: 'Connection ID'
	String get mobileConnectionId => 'Connection ID';

	/// en-US: 'Device ID'
	String get mobileDeviceId => 'Device ID';

	/// en-US: 'Account ID'
	String get mobileAccountId => 'Account ID';

	/// en-US: 'Account'
	String get mobileAccount => 'Account';

	/// en-US: 'Account name'
	String get mobileAccountName => 'Account name';

	/// en-US: 'Account details'
	String get mobileAccountDetails => 'Account details';

	/// en-US: 'Account result'
	String get mobileAccountResult => 'Account result';

	/// en-US: 'Appliance platforms'
	String get mobileAppliancePlatforms => 'Appliance platforms';

	/// en-US: 'Meiju sign-in'
	String get mobileMeijuSignIn => 'Meiju sign-in';

	/// en-US: 'Sign in to Meiju'
	String get mobileSignInToMeiju => 'Sign in to Meiju';

	/// en-US: 'Connect Meiju'
	String get mobileConnectMeiju => 'Connect Meiju';

	/// en-US: 'Choose Meiju'
	String get mobileChooseMeiju => 'Choose Meiju';

	/// en-US: 'Continue to Sign In'
	String get mobileContinueToSignIn => 'Continue to Sign In';

	/// en-US: 'Save Account Name'
	String get mobileSaveAccountName => 'Save Account Name';

	/// en-US: 'Reload Accounts'
	String get mobileReloadAccounts => 'Reload Accounts';

	/// en-US: 'Retrieve Devices'
	String get mobileRetrieveDevices => 'Retrieve Devices';

	/// en-US: 'Retrieve Saved Accounts'
	String get mobileRetrieveSavedAccounts => 'Retrieve Saved Accounts';

	/// en-US: 'Recover Saved Authorization'
	String get mobileRecoverSavedAuthorization => 'Recover Saved Authorization';

	/// en-US: 'Sign In to the Original Account'
	String get mobileSignInToTheOriginalAccount => 'Sign In to the Original Account';

	/// en-US: 'Reauthorize'
	String get mobileReauthorize => 'Reauthorize';

	/// en-US: 'Rename'
	String get mobileRename => 'Rename';

	/// en-US: 'Remove'
	String get mobileRemove => 'Remove';

	/// en-US: 'Remove Account'
	String get mobileRemoveAccount => 'Remove Account';

	/// en-US: 'Remove Platform'
	String get mobileRemovePlatform => 'Remove Platform';

	/// en-US: 'Reload local data'
	String get mobileReloadLocalData => 'Reload local data';

	/// en-US: 'Inspect Saved Inventory'
	String get mobileInspectSavedInventory => 'Inspect Saved Inventory';

	/// en-US: 'Inspect local permission'
	String get mobileInspectLocalPermission => 'Inspect local permission';

	/// en-US: 'Review control options'
	String get mobileReviewControlOptions => 'Review control options';

	/// en-US: 'Does not enable controls'
	String get mobileDoesNotEnableControls => 'Does not enable controls';

	/// en-US: 'Allow controls for this session'
	String get mobileAllowControlsForThisSession => 'Allow controls for this session';

	/// en-US: 'Allow Selected Controls'
	String get mobileAllowSelectedControls => 'Allow Selected Controls';

	/// en-US: 'Review Selected Permissions'
	String get mobileReviewSelectedPermissions => 'Review Selected Permissions';

	/// en-US: 'Change Selection'
	String get mobileChangeSelection => 'Change Selection';

	/// en-US: 'Revoke Session Controls'
	String get mobileRevokeSessionControls => 'Revoke Session Controls';

	/// en-US: 'Reopen Read-only'
	String get mobileReopenReadOnly => 'Reopen Read-only';

	/// en-US: 'Retrieve Control Capabilities'
	String get mobileRetrieveControlCapabilities => 'Retrieve Control Capabilities';

	/// en-US: 'Power on'
	String get mobilePowerOn => 'Power on';

	/// en-US: 'Power off'
	String get mobilePowerOff => 'Power off';

	/// en-US: 'Powered on'
	String get mobilePoweredOn => 'Powered on';

	/// en-US: 'Powered off'
	String get mobilePoweredOff => 'Powered off';

	/// en-US: 'Turning on\u2026'
	String get mobileTurningOn => 'Turning on\u2026';

	/// en-US: 'Turning off\u2026'
	String get mobileTurningOff => 'Turning off\u2026';

	/// en-US: 'Power'
	String get mobilePower => 'Power';

	/// en-US: 'Cooling'
	String get mobileCooling => 'Cooling';

	/// en-US: 'Heating'
	String get mobileHeating => 'Heating';

	/// en-US: 'Current temperature'
	String get mobileCurrentTemperature => 'Current temperature';

	/// en-US: 'Target'
	String get mobileTarget => 'Target';

	/// en-US: 'OBSERVED TARGET'
	String get mobileObservedTarget => 'OBSERVED TARGET';

	/// en-US: 'Refresh State'
	String get mobileRefreshState => 'Refresh State';

	/// en-US: 'Recent Operations'
	String get mobileRecentOperations => 'Recent Operations';

	/// en-US: 'View Operations'
	String get mobileViewOperations => 'View Operations';

	/// en-US: 'Device Information'
	String get mobileDeviceInformation => 'Device Information';

	/// en-US: 'Device information'
	String get mobileDeviceInformationDf743d => 'Device information';

	/// en-US: 'Device'
	String get mobileDevice => 'Device';

	/// en-US: 'Local name'
	String get mobileLocalName => 'Local name';

	/// en-US: 'Save Name'
	String get mobileSaveName => 'Save Name';

	/// en-US: 'Move to Room'
	String get mobileMoveToRoom => 'Move to Room';

	/// en-US: 'Pin to Top'
	String get mobilePinToTop => 'Pin to Top';

	/// en-US: 'Unpin'
	String get mobileUnpin => 'Unpin';

	/// en-US: 'Pinned'
	String get mobilePinned => 'Pinned';

	/// en-US: 'Reorder Devices'
	String get mobileReorderDevices => 'Reorder Devices';

	/// en-US: 'Select Devices'
	String get mobileSelectDevices => 'Select Devices';

	/// en-US: 'Selected Devices'
	String get mobileSelectedDevices => 'Selected Devices';

	/// en-US: 'Manage Rooms'
	String get mobileManageRooms => 'Manage Rooms';

	/// en-US: 'Rename Room'
	String get mobileRenameRoom => 'Rename Room';

	/// en-US: 'Delete Room'
	String get mobileDeleteRoom => 'Delete Room';

	/// en-US: 'Delete'
	String get mobileDelete => 'Delete';

	/// en-US: 'Move devices'
	String get mobileMoveDevices => 'Move devices';

	/// en-US: 'Move up'
	String get mobileMoveUp => 'Move up';

	/// en-US: 'Move down'
	String get mobileMoveDown => 'Move down';

	/// en-US: 'Search devices'
	String get mobileSearchDevices => 'Search devices';

	/// en-US: 'Search by name'
	String get mobileSearchByName => 'Search by name';

	/// en-US: 'Clear search'
	String get mobileClearSearch => 'Clear search';

	/// en-US: 'Add a platform to find your appliances.'
	String get mobileAddAPlatformToFindYourAppliances => 'Add a platform to find your appliances.';

	/// en-US: 'Choose a Platform'
	String get mobileChooseAPlatform => 'Choose a Platform';

	/// en-US: 'Choose Devices'
	String get mobileChooseDevices => 'Choose Devices';

	/// en-US: 'Choose Mode'
	String get mobileChooseMode => 'Choose Mode';

	/// en-US: 'Choose values \u203a'
	String get mobileChooseValues => 'Choose values  \u203a';

	/// en-US: 'Device organization'
	String get mobileDeviceOrganization => 'Device organization';

	/// en-US: 'Local data'
	String get mobileLocalData => 'Local data';

	/// en-US: 'Clear Cached State'
	String get mobileClearCachedState => 'Clear Cached State';

	/// en-US: 'Clear Recent Operations'
	String get mobileClearRecentOperations => 'Clear Recent Operations';

	/// en-US: 'Confirm Clear'
	String get mobileConfirmClear => 'Confirm Clear';

	/// en-US: 'Clear cached state?'
	String get mobileClearCachedStateC27869 => 'Clear cached state?';

	/// en-US: 'Clear recent operations?'
	String get mobileClearRecentOperationsA8d1ab => 'Clear recent operations?';

	/// en-US: 'Cached device state'
	String get mobileCachedDeviceState => 'Cached device state';

	/// en-US: 'Local cleanup completed'
	String get mobileLocalCleanupCompleted => 'Local cleanup completed';

	/// en-US: 'Local cleanup cancelled'
	String get mobileLocalCleanupCancelled => 'Local cleanup cancelled';

	/// en-US: 'Check this value'
	String get mobileCheckThisValue => 'Check this value';

	/// en-US: 'Could not complete this operation'
	String get mobileCouldNotCompleteThisOperation => 'Could not complete this operation';

	/// en-US: 'Dismiss error'
	String get mobileDismissError => 'Dismiss error';

	/// en-US: 'Native Core unavailable'
	String get mobileNativeCoreUnavailable => 'Native Core unavailable';

	/// en-US: 'Unavailable'
	String get mobileUnavailable => 'Unavailable';

	/// en-US: 'Unsupported'
	String get mobileUnsupported => 'Unsupported';

	/// en-US: 'Unsupported model'
	String get mobileUnsupportedModel => 'Unsupported model';

	/// en-US: 'Not read'
	String get mobileNotRead => 'Not read';

	/// en-US: 'Saved observation'
	String get mobileSavedObservation => 'Saved observation';

	/// en-US: 'Not provided'
	String get mobileNotProvided => 'Not provided';

	/// en-US: 'Accepted'
	String get mobileAccepted => 'Accepted';

	/// en-US: 'Verified'
	String get mobileVerified => 'Verified';

	/// en-US: 'Partially completed'
	String get mobilePartiallyCompleted => 'Partially completed';

	/// en-US: 'Request'
	String get mobileRequest => 'Request';

	/// en-US: 'Started'
	String get mobileStarted => 'Started';

	/// en-US: 'Recorded'
	String get mobileRecorded => 'Recorded';

	/// en-US: 'Error'
	String get mobileError => 'Error';

	/// en-US: 'Source'
	String get mobileSource => 'Source';

	/// en-US: 'Method'
	String get mobileMethod => 'Method';

	/// en-US: 'Observed'
	String get mobileObserved => 'Observed';

	/// en-US: 'Received'
	String get mobileReceived => 'Received';

	/// en-US: 'Verification evidence'
	String get mobileVerificationEvidence => 'Verification evidence';

	/// en-US: 'Correlated completion'
	String get mobileCorrelatedCompletion => 'Correlated completion';

	/// en-US: 'State match'
	String get mobileStateMatch => 'State match';

	/// en-US: 'Returned values'
	String get mobileReturnedValues => 'Returned values';

	/// en-US: 'Recorded attempts'
	String get mobileRecordedAttempts => 'Recorded attempts';

	/// en-US: 'Sub-operations'
	String get mobileSubOperations => 'Sub-operations';

	/// en-US: 'Not sent'
	String get mobileNotSent => 'Not sent';

	/// en-US: 'May have been sent'
	String get mobileMayHaveBeenSent => 'May have been sent';

	/// en-US: 'Rejected'
	String get mobileRejected => 'Rejected';

	/// en-US: 'Unknown phase'
	String get mobileUnknownPhase => 'Unknown phase';

	/// en-US: 'No attempts are recorded.'
	String get mobileNoAttemptsAreRecorded => 'No attempts are recorded.';

	/// en-US: 'No retained receipt selected.'
	String get mobileNoRetainedReceiptSelected => 'No retained receipt selected.';

	/// en-US: 'No retained operations.'
	String get mobileNoRetainedOperations => 'No retained operations.';

	/// en-US: 'No devices in this view.'
	String get mobileNoDevicesInThisView => 'No devices in this view.';

	/// en-US: 'No devices'
	String get mobileNoDevices => 'No devices';

	/// en-US: 'No devices yet'
	String get mobileNoDevicesYet => 'No devices yet';

	/// en-US: 'No rooms yet'
	String get mobileNoRoomsYet => 'No rooms yet';

	/// en-US: 'No accounts yet'
	String get mobileNoAccountsYet => 'No accounts yet';

	/// en-US: 'Loading local devices\u2026'
	String get mobileLoadingLocalDevices => 'Loading local devices\u2026';

	/// en-US: 'Mock discovery'
	String get mobileMockDiscovery => 'Mock discovery';

	/// en-US: 'Saved-account retrieval'
	String get mobileSavedAccountRetrieval => 'Saved-account retrieval';

	/// en-US: 'Discover Mock device'
	String get mobileDiscoverMockDevice => 'Discover Mock device';

	/// en-US: 'Mock tested'
	String get mobileMockTested => 'Mock tested';

	/// en-US: 'Compatibility'
	String get mobileCompatibility => 'Compatibility';

	/// en-US: 'Compatibility Details'
	String get mobileCompatibilityDetails => 'Compatibility Details';

	/// en-US: 'Privacy'
	String get mobilePrivacy => 'Privacy';

	/// en-US: 'Licenses'
	String get mobileLicenses => 'Licenses';

	/// en-US: 'Preview Diagnostics'
	String get mobilePreviewDiagnostics => 'Preview Diagnostics';

	/// en-US: 'Export Diagnostics'
	String get mobileExportDiagnostics => 'Export Diagnostics';

	/// en-US: 'Save Report'
	String get mobileSaveReport => 'Save Report';

	/// en-US: 'App version'
	String get mobileAppVersion => 'App version';

	/// en-US: 'Current skin'
	String get mobileCurrentSkin => 'Current skin';

	/// en-US: 'Current'
	String get mobileCurrent => 'Current';

	/// en-US: 'Window Light'
	String get mobileWindowLight => 'Window Light';

	/// en-US: 'Design preview'
	String get mobileDesignPreview => 'Design preview';

	/// en-US: 'Preview catalog'
	String get mobilePreviewCatalog => 'Preview catalog';

	/// en-US: 'Close catalog'
	String get mobileCloseCatalog => 'Close catalog';

	/// en-US: 'Authorization sharing'
	String get mobileAuthorizationSharing => 'Authorization sharing';

	/// en-US: 'Sharing name'
	String get mobileSharingName => 'Sharing name';

	/// en-US: 'Send Authorization'
	String get mobileSendAuthorization => 'Send Authorization';

	/// en-US: 'Receive Authorization'
	String get mobileReceiveAuthorization => 'Receive Authorization';

	/// en-US: 'Platform connection'
	String get mobilePlatformConnection => 'Platform connection';

	/// en-US: 'Platform'
	String get mobilePlatform => 'Platform';

	/// en-US: 'Authentication'
	String get mobileAuthentication => 'Authentication';

	/// en-US: 'Authorization check'
	String get mobileAuthorizationCheck => 'Authorization check';

	/// en-US: 'Credential storage'
	String get mobileCredentialStorage => 'Credential storage';

	/// en-US: 'Saved devices'
	String get mobileSavedDevices => 'Saved devices';

	/// en-US: 'Device retrieval complete'
	String get mobileDeviceRetrievalComplete => 'Device retrieval complete';

	/// en-US: 'Device retrieval incomplete'
	String get mobileDeviceRetrievalIncomplete => 'Device retrieval incomplete';

	/// en-US: 'Device retrieval failed'
	String get mobileDeviceRetrievalFailed => 'Device retrieval failed';

	/// en-US: 'Authorization needs recovery'
	String get mobileAuthorizationNeedsRecovery => 'Authorization needs recovery';

	/// en-US: 'Authorization expired'
	String get mobileAuthorizationExpired => 'Authorization expired';

	/// en-US: 'Sign-in needed'
	String get mobileSignInNeeded => 'Sign-in needed';

	/// en-US: 'Local account details'
	String get mobileLocalAccountDetails => 'Local account details';

	/// en-US: 'Account connection is unavailable in this Mock build.'
	String get mobileAccountConnectionIsUnavailableInThisMockBuild => 'Account connection is unavailable in this Mock build.';

	/// en-US: 'Connecting and retrieving devices\u2026'
	String get mobileConnectingAndRetrievingDevices => 'Connecting and retrieving devices\u2026';

	/// en-US: 'Component'
	String get mobileComponent => 'Component';

	/// en-US: 'Exact constraints'
	String get mobileExactConstraints => 'Exact constraints';

	/// en-US: 'Control'
	String get mobileControl => 'Control';

	/// en-US: 'Controls'
	String get mobileControls => 'Controls';

	/// en-US: 'Control unavailable.'
	String get mobileControlUnavailable => 'Control unavailable.';

	/// en-US: 'Action unavailable'
	String get mobileActionUnavailable => 'Action unavailable';

	/// en-US: 'Review Command'
	String get mobileReviewCommand => 'Review Command';

	/// en-US: 'Confirm Command'
	String get mobileConfirmCommand => 'Confirm Command';

	/// en-US: 'Look Up Receipt'
	String get mobileLookUpReceipt => 'Look Up Receipt';

	/// en-US: 'Connection \u00b7 Required'
	String get mobileConnectionRequired => 'Connection \u00b7 Required';

	/// en-US: 'Brightness'
	String get mobileBrightness => 'Brightness';

	/// en-US: 'Color temperature'
	String get mobileColorTemperature => 'Color temperature';

	/// en-US: 'Speed'
	String get mobileSpeed => 'Speed';

	/// en-US: 'Oscillation'
	String get mobileOscillation => 'Oscillation';

	/// en-US: 'Muted'
	String get mobileMuted => 'Muted';

	/// en-US: 'Volume'
	String get mobileVolume => 'Volume';

	/// en-US: 'Source input'
	String get mobileSourceInput => 'Source input';

	/// en-US: 'Playback'
	String get mobilePlayback => 'Playback';

	/// en-US: 'Play'
	String get mobilePlay => 'Play';

	/// en-US: 'Pause'
	String get mobilePause => 'Pause';

	/// en-US: 'Resume'
	String get mobileResume => 'Resume';

	/// en-US: 'Stop'
	String get mobileStop => 'Stop';

	/// en-US: 'Cycle'
	String get mobileCycle => 'Cycle';

	/// en-US: 'Program'
	String get mobileProgram => 'Program';

	/// en-US: 'Duration'
	String get mobileDuration => 'Duration';

	/// en-US: 'Remaining time'
	String get mobileRemainingTime => 'Remaining time';

	/// en-US: 'Closed'
	String get mobileClosed => 'Closed';

	/// en-US: 'Active'
	String get mobileActive => 'Active';

	/// en-US: 'Stale'
	String get mobileStale => 'Stale';

	/// en-US: 'Cached'
	String get mobileCached => 'Cached';

	/// en-US: 'Read-only'
	String get mobileReadOnly => 'Read-only';

	/// en-US: 'Read only'
	String get mobileReadOnly8ac767 => 'Read only';

	/// en-US: 'Refresh'
	String get mobileRefresh => 'Refresh';

	/// en-US: 'Retry'
	String get mobileRetry => 'Retry';

	/// en-US: 'Confirm'
	String get mobileConfirm => 'Confirm';

	/// en-US: 'Done'
	String get mobileDone => 'Done';

	/// en-US: 'Next'
	String get mobileNext => 'Next';

	/// en-US: 'Not Now'
	String get mobileNotNow => 'Not Now';

	/// en-US: 'No'
	String get mobileNo => 'No';

	/// en-US: 'Yes'
	String get mobileYes => 'Yes';

	/// en-US: 'Details unavailable'
	String get mobileDetailsUnavailable => 'Details unavailable';

	/// en-US: 'Complete'
	String get mobileComplete => 'Complete';

	/// en-US: 'Partial'
	String get mobilePartial => 'Partial';

	/// en-US: 'Saved'
	String get mobileSaved => 'Saved';

	/// en-US: 'Stored'
	String get mobileStored => 'Stored';

	/// en-US: 'Pending'
	String get mobilePending => 'Pending';

	/// en-US: 'Removing'
	String get mobileRemoving => 'Removing';

	/// en-US: 'Connected'
	String get mobileConnected => 'Connected';

	/// en-US: 'Disconnected'
	String get mobileDisconnected => 'Disconnected';

	/// en-US: 'GENERAL'
	String get mobileGeneral => 'GENERAL';

	/// en-US: 'DATA & SUPPORT'
	String get mobileDataSupport => 'DATA & SUPPORT';

	/// en-US: 'AUTHORIZATION SHARING'
	String get mobileAuthorizationSharing4e5636 => 'AUTHORIZATION SHARING';

	/// en-US: 'Allow'
	String get mobileAllow => 'Allow';

	/// en-US: 'Allowed'
	String get mobileAllowed => 'Allowed';

	/// en-US: 'Back to Accounts'
	String get mobileBackToAccounts => 'Back to Accounts';

	/// en-US: 'Back to Platforms'
	String get mobileBackToPlatforms => 'Back to Platforms';

	/// en-US: 'Back to Appearance'
	String get mobileBackToAppearance => 'Back to Appearance';

	/// en-US: 'Go to Platforms'
	String get mobileGoToPlatforms => 'Go to Platforms';

	/// en-US: 'Open Platforms'
	String get mobileOpenPlatforms => 'Open Platforms';

	/// en-US: 'Local network only \u2014 no cloud fallback'
	String get mobileLocalNetworkOnlyNoCloudFallback => 'Local network only \u2014 no cloud fallback';

	/// en-US: 'Platform cloud \u2014 requires internet'
	String get mobilePlatformCloudRequiresInternet => 'Platform cloud \u2014 requires internet';

	/// en-US: 'Auto \u2014 prefer local when available'
	String get mobileAutoPreferLocalWhenAvailable => 'Auto \u2014 prefer local when available';

	/// en-US: '0.2.0 \u00b7 Native Mock host \u00b7 Fictional devices'
	String get mobile020NativeMockHostFictionalDevices => '0.2.0 \u00b7 Native Mock host \u00b7 Fictional devices';

	/// en-US: '0.2.0 \u00b7 Experimental Meiju \u00b7 Read-only session'
	String get mobile020ExperimentalMeijuReadOnlySession => '0.2.0 \u00b7 Experimental Meiju \u00b7 Read-only session';

	/// en-US: '0.2.0 \u00b7 Experimental Meiju \u00b7 Scoped controls'
	String get mobile020ExperimentalMeijuScopedControls => '0.2.0 \u00b7 Experimental Meiju \u00b7 Scoped controls';

	/// en-US: 'One language for your appliances.'
	String get mobileOneLanguageForYourAppliances => 'One language for your appliances.';

	/// en-US: 'Simplified Chinese'
	String get mobileLanguageName => 'Simplified Chinese';

	/// en-US: 'Language changes only the interface. Device names, saved data and control permissions stay unchanged.'
	String get mobileLanguageNotice => 'Language changes only the interface. Device names, saved data and control permissions stay unchanged.';

	/// en-US: 'A blank name restores the adapter name.'
	String get mobileABlankNameRestoresTheAdapterName => 'A blank name restores the adapter name.';

	/// en-US: 'A device-specific preference takes priority. No background scanning.'
	String get mobileADeviceSpecificPreferenceTakesPriorityNoBackground => 'A device-specific preference takes priority. No background scanning.';

	/// en-US: 'A native library with API negotiation is required.'
	String get mobileANativeLibraryWithApiNegotiationIsRequired => 'A native library with API negotiation is required.';

	/// en-US: 'A new sign-in is required'
	String get mobileANewSignInIsRequired => 'A new sign-in is required';

	/// en-US: 'A room with this name already exists.'
	String get mobileARoomWithThisNameAlreadyExists => 'A room with this name already exists.';

	/// en-US: 'Account already exists'
	String get mobileAccountAlreadyExists => 'Account already exists';

	/// en-US: 'Accounts, credentials, devices, rooms, preferences and permissions remain. This does not securely erase backups or storage pages.'
	String get mobileAccountsCredentialsDevicesRoomsPreferencesAndPermissionsRemain => 'Accounts, credentials, devices, rooms, preferences and permissions remain. This does not securely erase backups or storage pages.';

	/// en-US: 'All listed arguments are explicitly set. Preparation sends nothing and does not check live preconditions. Confirmation executes once; an uncertain result must not be repeated automatically.'
	String get mobileAllListedArgumentsAreExplicitlySetPreparationSends => 'All listed arguments are explicitly set. Preparation sends nothing and does not check live preconditions. Confirmation executes once; an uncertain result must not be repeated automatically.';

	/// en-US: 'Apply to Selected Devices'
	String get mobileApplyToSelectedDevices => 'Apply to Selected Devices';

	/// en-US: 'Attempts omitted'
	String get mobileAttemptsOmitted => 'Attempts omitted';

	/// en-US: 'Authorizations are held in secure storage. Local removal does not revoke remote access.'
	String get mobileAuthorizationsAreHeldInSecureStorageLocalRemoval => 'Authorizations are held in secure storage. Local removal does not revoke remote access.';

	/// en-US: 'Choose On or Off.'
	String get mobileChooseOnOrOff => 'Choose On or Off.';

	/// en-US: 'Choose a supported action from the device page.'
	String get mobileChooseASupportedActionFromTheDevicePage => 'Choose a supported action from the device page.';

	/// en-US: 'Choose a supported value.'
	String get mobileChooseASupportedValue => 'Choose a supported value.';

	/// en-US: 'Choose exact devices and a power action. Each outcome is recorded separately.'
	String get mobileChooseExactDevicesAndAPowerActionEach => 'Choose exact devices and a power action. Each outcome is recorded separately.';

	/// en-US: 'Choose exact values for this device over the cloud. These experimental controls can affect the real appliance. Mode activation also turns it on. Permission lasts until the app process closes or you revoke it.'
	String get mobileChooseExactValuesForThisDeviceOverThe => 'Choose exact values for this device over the cloud. These experimental controls can affect the real appliance. Mode activation also turns it on. Permission lasts until the app process closes or you revoke it.';

	/// en-US: 'Choose one explicit connection.'
	String get mobileChooseOneExplicitConnection => 'Choose one explicit connection.';

	/// en-US: 'Choose only the values to send. Observations remain unchanged until a read.'
	String get mobileChooseOnlyTheValuesToSendObservationsRemain => 'Choose only the values to send. Observations remain unchanged until a read.';

	/// en-US: 'Choose the inspected cloud connection.'
	String get mobileChooseTheInspectedCloudConnection => 'Choose the inspected cloud connection.';

	/// en-US: 'Choose what to clear. Accounts, credentials, devices, rooms and preferences stay on this installation.'
	String get mobileChooseWhatToClearAccountsCredentialsDevicesRooms => 'Choose what to clear. Accounts, credentials, devices, rooms and preferences stay on this installation.';

	/// en-US: 'Cleanup was not fully confirmed. Reload local data before deciding what to do next.'
	String get mobileCleanupWasNotFullyConfirmedReloadLocalData => 'Cleanup was not fully confirmed. Reload local data before deciding what to do next.';

	/// en-US: 'Completion evidence does not establish the current state or exactly-once execution.'
	String get mobileCompletionEvidenceDoesNotEstablishTheCurrentState => 'Completion evidence does not establish the current state or exactly-once execution.';

	/// en-US: 'Component reading was cancelled.'
	String get mobileComponentReadingWasCancelled => 'Component reading was cancelled.';

	/// en-US: 'Confirm the exact action below. It may have lasting effects.'
	String get mobileConfirmTheExactActionBelowItMayHave => 'Confirm the exact action below. It may have lasting effects.';

	/// en-US: 'Core contract 1.9 includes experimental Meiju reads and protected account connection. Physical account and phone acceptance of this UI remain pending.'
	String get mobileCoreContract19IncludesExperimentalMeijuReads => 'Core contract 1.9 includes experimental Meiju reads and protected account connection. Physical account and phone acceptance of this UI remain pending.';

	/// en-US: 'Could not save the previewed report.'
	String get mobileCouldNotSaveThePreviewedReport => 'Could not save the previewed report.';

	/// en-US: 'Current local session permissions. Provider authorization and device availability are not checked. Core checks every command.'
	String get mobileCurrentLocalSessionPermissionsProviderAuthorizationAndDevice => 'Current local session permissions. Provider authorization and device availability are not checked. Core checks every command.';

	/// en-US: 'Current provider authorization has not been checked by this local view.'
	String get mobileCurrentProviderAuthorizationHasNotBeenCheckedBy => 'Current provider authorization has not been checked by this local view.';

	/// en-US: 'Current state matches the requested state.'
	String get mobileCurrentStateMatchesTheRequestedState => 'Current state matches the requested state.';

	/// en-US: 'Decrease'
	String get mobileDecrease => 'Decrease';

	/// en-US: 'Decrease target'
	String get mobileDecreaseTarget => 'Decrease target';

	/// en-US: 'Device information \u00b7 Recent operations'
	String get mobileDeviceInformationRecentOperations => 'Device information \u00b7 Recent operations';

	/// en-US: 'Device retrieval did not complete. Review the account result.'
	String get mobileDeviceRetrievalDidNotCompleteReviewTheAccount => 'Device retrieval did not complete. Review the account result.';

	/// en-US: 'Device state'
	String get mobileDeviceState => 'Device state';

	/// en-US: 'Device state is read only when requested.'
	String get mobileDeviceStateIsReadOnlyWhenRequested => 'Device state is read only when requested.';

	/// en-US: 'Device-specific preferences take priority. No background scanning.'
	String get mobileDeviceSpecificPreferencesTakePriorityNoBackgroundScanning => 'Device-specific preferences take priority. No background scanning.';

	/// en-US: 'Devices / rooms'
	String get mobileDevicesRooms => 'Devices / rooms';

	/// en-US: 'Devices added'
	String get mobileDevicesAdded => 'Devices added';

	/// en-US: 'Devices move to Unassigned. No devices or platform accounts are deleted.'
	String get mobileDevicesMoveToUnassignedNoDevicesOrPlatform => 'Devices move to Unassigned. No devices or platform accounts are deleted.';

	/// en-US: 'Diagnostic report'
	String get mobileDiagnosticReport => 'Diagnostic report';

	/// en-US: 'Direct cloud connection. Starts read-only; reviewed controls require separate permission.'
	String get mobileDirectCloudConnectionSupportedDevicesUseReadOnly => 'Direct cloud connection. Starts read-only; reviewed controls require separate permission.';

	/// en-US: 'Discovery'
	String get mobileDiscovery => 'Discovery';

	/// en-US: 'Dismiss'
	String get mobileDismiss => 'Dismiss';

	/// en-US: 'Door'
	String get mobileDoor => 'Door';

	/// en-US: 'Drag to reorder. Use a row menu to rename or delete.'
	String get mobileDragToReorderUseARowMenuTo => 'Drag to reorder. Use a row menu to rename or delete.';

	/// en-US: 'Enter a number.'
	String get mobileEnterANumber => 'Enter a number.';

	/// en-US: 'Estimate reported by this appliance.'
	String get mobileEstimateReportedByThisAppliance => 'Estimate reported by this appliance.';

	/// en-US: 'Estimated duration'
	String get mobileEstimatedDuration => 'Estimated duration';

	/// en-US: 'Existing cards are reloaded from local storage. Retrieval does not establish physical state or grant appliance controls.'
	String get mobileExistingCardsAreReloadedFromLocalStorageRetrieval => 'Existing cards are reloaded from local storage. Retrieval does not establish physical state or grant appliance controls.';

	/// en-US: 'Expiry'
	String get mobileExpiry => 'Expiry';

	/// en-US: 'Explicit account connection and device retrieval can contact the admitted provider directly. No telemetry or diagnostic uploads. Credentials use the system secure store. Diagnostics are exported only when requested. Nearby sharing is unavailable.'
	String get mobileExplicitAccountConnectionAndDeviceRetrievalCanContact => 'Explicit account connection and device retrieval can contact the admitted provider directly. No telemetry or diagnostic uploads. Credentials use the system secure store. Diagnostics are exported only when requested. Nearby sharing is unavailable.';

	/// en-US: 'Explicitly retrieve cloud inventory from saved Meiju accounts. No LAN scan or pairing is started.'
	String get mobileExplicitlyRetrieveCloudInventoryFromSavedMeijuAccounts => 'Explicitly retrieve cloud inventory from saved Meiju accounts. No LAN scan or pairing is started.';

	/// en-US: 'Export Redacted Report'
	String get mobileExportRedactedReport => 'Export Redacted Report';

	/// en-US: 'Fan only'
	String get mobileFanOnly => 'Fan only';

	/// en-US: 'Flow unavailable'
	String get mobileFlowUnavailable => 'Flow unavailable';

	/// en-US: 'Foreground session stopped'
	String get mobileForegroundSessionStopped => 'Foreground session stopped';

	/// en-US: 'Freezer'
	String get mobileFreezer => 'Freezer';

	/// en-US: 'Fridge'
	String get mobileFridge => 'Fridge';

	/// en-US: 'Full cleanup unavailable'
	String get mobileFullCleanupUnavailable => 'Full cleanup unavailable';

	/// en-US: 'Full-profile deletion is not available. Use the separate cache or history action.'
	String get mobileFullProfileDeletionIsNotAvailableUseThe => 'Full-profile deletion is not available. Use the separate cache or history action.';

	/// en-US: 'Full-profile deletion is unavailable. Backups and storage pages are not securely erased.'
	String get mobileFullProfileDeletionIsUnavailableBackupsAndStorage => 'Full-profile deletion is unavailable. Backups and storage pages are not securely erased.';

	/// en-US: 'Generated'
	String get mobileGenerated => 'Generated';

	/// en-US: 'Home account'
	String get mobileHomeAccount => 'Home account';

	/// en-US: 'Home phone'
	String get mobileHomePhone => 'Home phone';

	/// en-US: 'Humidity'
	String get mobileHumidity => 'Humidity';

	/// en-US: 'Identity merge unavailable'
	String get mobileIdentityMergeUnavailable => 'Identity merge unavailable';

	/// en-US: 'Include every visible device exactly once.'
	String get mobileIncludeEveryVisibleDeviceExactlyOnce => 'Include every visible device exactly once.';

	/// en-US: 'Increase'
	String get mobileIncrease => 'Increase';

	/// en-US: 'Increase target'
	String get mobileIncreaseTarget => 'Increase target';

	/// en-US: 'Inspect cached state'
	String get mobileInspectCachedState => 'Inspect cached state';

	/// en-US: 'Inspect each item. Completed actions are not rolled back.'
	String get mobileInspectEachItemCompletedActionsAreNotRolled => 'Inspect each item. Completed actions are not rolled back.';

	/// en-US: 'Install the matching native library and restart. No device or account operation was started.'
	String get mobileInstallTheMatchingNativeLibraryAndRestartNo => 'Install the matching native library and restart. No device or account operation was started.';

	/// en-US: 'Isolated preview scenarios'
	String get mobileIsolatedPreviewScenarios => 'Isolated preview scenarios';

	/// en-US: 'Just now'
	String get mobileJustNow => 'Just now';

	/// en-US: 'LAN / cloud routes'
	String get mobileLanCloudRoutes => 'LAN / cloud routes';

	/// en-US: 'Last checked'
	String get mobileLastChecked => 'Last checked';

	/// en-US: 'Last-read snapshots'
	String get mobileLastReadSnapshots => 'Last-read snapshots';

	/// en-US: 'Latest 100 records'
	String get mobileLatest100Records => 'Latest 100 records';

	/// en-US: 'Latest receipt'
	String get mobileLatestReceipt => 'Latest receipt';

	/// en-US: 'Linked cards'
	String get mobileLinkedCards => 'Linked cards';

	/// en-US: 'Listed in the last complete inventory'
	String get mobileListedInTheLastCompleteInventory => 'Listed in the last complete inventory';

	/// en-US: 'Loading local accounts\u2026'
	String get mobileLoadingLocalAccounts => 'Loading local accounts\u2026';

	/// en-US: 'Local Data'
	String get mobileLocalData681b68 => 'Local Data';

	/// en-US: 'Local account management requires a native library update.'
	String get mobileLocalAccountManagementRequiresANativeLibraryUpdate => 'Local account management requires a native library update.';

	/// en-US: 'Local cleanup pending'
	String get mobileLocalCleanupPending => 'Local cleanup pending';

	/// en-US: 'Local controls are allowed for this session. Retrieve this account to update available actions. Each command is checked independently.'
	String get mobileLocalControlsAreAllowedForThisSessionRetrieve => 'Local controls are allowed for this session. Retrieve this account to update available actions. Each command is checked independently.';

	/// en-US: 'Local removal does not revoke provider access or copies on another installation.'
	String get mobileLocalRemovalDoesNotRevokeProviderAccessOr => 'Local removal does not revoke provider access or copies on another installation.';

	/// en-US: 'Local scope'
	String get mobileLocalScope => 'Local scope';

	/// en-US: 'Mainland China'
	String get mobileMainlandChina => 'Mainland China';

	/// en-US: 'Meiju account'
	String get mobileMeijuAccount => 'Meiju account';

	/// en-US: 'Password'
	String get mobilePassword => 'Password';

	/// en-US: 'Sign In and Retrieve Devices'
	String get mobileSignInAndRetrieveDevices => 'Sign In and Retrieve Devices';

	/// en-US: 'Sign-in connects directly to the admitted mainland provider and automatically retrieves devices. Saved authorization uses the system secure store.'
	String get mobileSignInConnectsDirectlyToTheAdmittedMainland => 'Sign-in connects directly to the admitted mainland provider and automatically retrieves devices. Saved authorization uses the system secure store.';

	/// en-US: 'Reload saved accounts before starting another connection. An interrupted sign-in may have saved authorization.'
	String get mobileReloadSavedAccountsBeforeStartingAnotherConnectionAn => 'Reload saved accounts before starting another connection. An interrupted sign-in may have saved authorization.';

	/// en-US: 'Only these selected values will be allowed:'
	String get mobileOnlyTheseSelectedValuesWillBeAllowed => 'Only these selected values will be allowed:';

	/// en-US: 'This changes local permission only. After confirming, explicitly retrieve the account devices to publish enabled controls. Nothing is sent to the appliance now.'
	String get mobileThisChangesLocalPermissionOnlyAfterConfirmingExplicitly => 'This changes local permission only. After confirming, explicitly retrieve the account devices to publish enabled controls. Nothing is sent to the appliance now.';

	/// en-US: 'No reviewed controls can be enabled for this connection.'
	String get mobileNoReviewedControlsCanBeEnabledForThis => 'No reviewed controls can be enabled for this connection.';

	/// en-US: 'Reviewed saved model options. Permission is not checked and no controls are enabled.'
	String get mobileReviewedSavedModelOptionsPermissionIsNotChecked => 'Reviewed saved model options. Permission is not checked and no controls are enabled.';

	/// en-US: 'No controls are listed for this connection.'
	String get mobileNoControlsAreListedForThisConnection => 'No controls are listed for this connection.';

	/// en-US: 'Searches local device names, models and rooms.'
	String get mobileSearchesLocalDeviceNamesModelsAndRooms => 'Searches local device names, models and rooms.';

	/// en-US: 'The local device list could not be loaded.'
	String get mobileTheLocalDeviceListCouldNotBeLoaded => 'The local device list could not be loaded.';

	/// en-US: 'Nearby sharing is unavailable in the current native contract.'
	String get mobileNearbySharingIsUnavailableInTheCurrentNative => 'Nearby sharing is unavailable in the current native contract.';

	/// en-US: 'No discovery, transfer or authorization has started.'
	String get mobileNoDiscoveryTransferOrAuthorizationHasStarted => 'No discovery, transfer or authorization has started.';

	/// en-US: 'This changes only the local display name. Identity, credentials and permissions stay unchanged.'
	String get mobileThisChangesOnlyTheLocalDisplayNameIdentity => 'This changes only the local display name. Identity, credentials and permissions stay unchanged.';

	/// en-US: 'Use 1\u201380 characters without control characters.'
	String get mobileUse180CharactersWithoutControlCharacters => 'Use 1\u201380 characters without control characters.';

	/// en-US: 'No verification evidence is recorded for this outcome.'
	String get mobileNoVerificationEvidenceIsRecordedForThisOutcome => 'No verification evidence is recorded for this outcome.';

	/// en-US: 'Matching state alone does not establish physical causality.'
	String get mobileMatchingStateAloneDoesNotEstablishPhysicalCausality => 'Matching state alone does not establish physical causality.';

	/// en-US: 'Request accepted; completion has not been verified.'
	String get mobileRequestAcceptedCompletionHasNotBeenVerified => 'Request accepted; completion has not been verified.';

	/// en-US: 'Verification evidence was recorded for this operation.'
	String get mobileVerificationEvidenceWasRecordedForThisOperation => 'Verification evidence was recorded for this operation.';

	/// en-US: 'The request failed. Inspect its recorded attempts.'
	String get mobileTheRequestFailedInspectItsRecordedAttempts => 'The request failed. Inspect its recorded attempts.';

	/// en-US: 'This request may have executed. Refresh before considering another write.'
	String get mobileThisRequestMayHaveExecutedRefreshBeforeConsidering => 'This request may have executed. Refresh before considering another write.';

	/// en-US: 'Outcomes differ. Completed actions are not rolled back.'
	String get mobileOutcomesDifferCompletedActionsAreNotRolledBack => 'Outcomes differ. Completed actions are not rolled back.';

	/// en-US: 'No recognized outcome is available.'
	String get mobileNoRecognizedOutcomeIsAvailable => 'No recognized outcome is available.';

	/// en-US: 'Refresh reads device state; it never repeats this write or changes the retained outcome.'
	String get mobileRefreshReadsDeviceStateItNeverRepeatsThis => 'Refresh reads device state; it never repeats this write or changes the retained outcome.';

	/// en-US: 'Operation in progress'
	String get mobileOperationInProgress => 'Operation in progress';

	/// en-US: 'This action has no parameters.'
	String get mobileThisActionHasNoParameters => 'This action has no parameters.';

	/// en-US: 'This action may have lasting effects. An uncertain result must not be blindly repeated.'
	String get mobileThisActionMayHaveLastingEffectsAnUncertain => 'This action may have lasting effects. An uncertain result must not be blindly repeated.';

	/// en-US: 'Trusted approval is unavailable.'
	String get mobileTrustedApprovalIsUnavailable => 'Trusted approval is unavailable.';

	/// en-US: 'This form is not supported.'
	String get mobileThisFormIsNotSupported => 'This form is not supported.';

	/// en-US: 'This property is read-only.'
	String get mobileThisPropertyIsReadOnly => 'This property is read-only.';

	/// en-US: 'The confirmation reply was interrupted. Look up the original receipt before considering another command.'
	String get mobileTheConfirmationReplyWasInterruptedLookUpThe => 'The confirmation reply was interrupted. Look up the original receipt before considering another command.';

	/// en-US: 'No physical appliance or local network is scanned.'
	String get mobileNoPhysicalApplianceOrLocalNetworkIsScanned => 'No physical appliance or local network is scanned.';

	/// en-US: 'Only a fictional appliance can be discovered by this host.'
	String get mobileOnlyAFictionalApplianceCanBeDiscoveredBy => 'Only a fictional appliance can be discovered by this host.';

	/// en-US: 'New accounts connect from Add Platform and retrieve their devices automatically.'
	String get mobileNewAccountsConnectFromAddPlatformAndRetrieve => 'New accounts connect from Add Platform and retrieve their devices automatically.';

	/// en-US: 'Names alone do not prove identity. No native merge method exists.'
	String get mobileNamesAloneDoNotProveIdentityNoNative => 'Names alone do not prove identity. No native merge method exists.';

	/// en-US: 'Support requires model, region and physical-device evidence.'
	String get mobileSupportRequiresModelRegionAndPhysicalDeviceEvidence => 'Support requires model, region and physical-device evidence.';

	/// en-US: 'Shown only during an explicit nearby sharing session.'
	String get mobileShownOnlyDuringAnExplicitNearbySharingSession => 'Shown only during an explicit nearby sharing session.';

	/// en-US: 'Mainland China \u00b7 experimental'
	String get mobileMainlandChinaExperimental => 'Mainland China \u00b7 experimental';

	/// en-US: 'Make room for the devices you already have.'
	String get mobileMakeRoomForTheDevicesYouAlreadyHave => 'Make room for the devices you already have.';

	/// en-US: 'Meiju'
	String get mobileMeiju => 'Meiju';

	/// en-US: 'Meiju \u00b7 Mainland China'
	String get mobileMeijuMainlandChina => 'Meiju \u00b7 Mainland China';

	/// en-US: 'Membership unconfirmed'
	String get mobileMembershipUnconfirmed => 'Membership unconfirmed';

	/// en-US: 'Mock devices'
	String get mobileMockDevices => 'Mock devices';

	/// en-US: 'Mock profile'
	String get mobileMockProfile => 'Mock profile';

	/// en-US: 'More controls'
	String get mobileMoreControls => 'More controls';

	/// en-US: 'Move Devices'
	String get mobileMoveDevices9f8d41 => 'Move Devices';

	/// en-US: 'Move Existing Devices'
	String get mobileMoveExistingDevices => 'Move Existing Devices';

	/// en-US: 'My home'
	String get mobileMyHome => 'My home';

	/// en-US: 'Names alone do not prove a match.'
	String get mobileNamesAloneDoNotProveAMatch => 'Names alone do not prove a match.';

	/// en-US: 'Native API major 1 is required.'
	String get mobileNativeApiMajor1IsRequired => 'Native API major 1 is required.';

	/// en-US: 'Nearby Devices'
	String get mobileNearbyDevices => 'Nearby Devices';

	/// en-US: 'Nearby devices'
	String get mobileNearbyDevices500f1e => 'Nearby devices';

	/// en-US: 'Next step'
	String get mobileNextStep => 'Next step';

	/// en-US: 'No HomeVerb account needed.'
	String get mobileNoHomeverbAccountNeeded => 'No HomeVerb account needed.';

	/// en-US: 'No attempts'
	String get mobileNoAttempts => 'No attempts';

	/// en-US: 'No complete device-list time recorded.'
	String get mobileNoCompleteDeviceListTimeRecorded => 'No complete device-list time recorded.';

	/// en-US: 'No completed account report is available.'
	String get mobileNoCompletedAccountReportIsAvailable => 'No completed account report is available.';

	/// en-US: 'No current reading'
	String get mobileNoCurrentReading => 'No current reading';

	/// en-US: 'No devices found'
	String get mobileNoDevicesFound => 'No devices found';

	/// en-US: 'No equal saved device identifiers were found.'
	String get mobileNoEqualSavedDeviceIdentifiersWereFound => 'No equal saved device identifiers were found.';

	/// en-US: 'No new sign-in was attempted.'
	String get mobileNoNewSignInWasAttempted => 'No new sign-in was attempted.';

	/// en-US: 'No platform account is saved on this installation.'
	String get mobileNoPlatformAccountIsSavedOnThisInstallation => 'No platform account is saved on this installation.';

	/// en-US: 'No reading'
	String get mobileNoReading => 'No reading';

	/// en-US: 'No recent operations'
	String get mobileNoRecentOperations => 'No recent operations';

	/// en-US: 'No report loaded.'
	String get mobileNoReportLoaded => 'No report loaded.';

	/// en-US: 'No response'
	String get mobileNoResponse => 'No response';

	/// en-US: 'No retained operations. This does not prove that no past action occurred.'
	String get mobileNoRetainedOperationsThisDoesNotProveThat => 'No retained operations. This does not prove that no past action occurred.';

	/// en-US: 'No saved bindings were returned. This does not prove the cloud account is empty. Use Retrieve Devices to refresh explicitly.'
	String get mobileNoSavedBindingsWereReturnedThisDoesNot => 'No saved bindings were returned. This does not prove the cloud account is empty. Use Retrieve Devices to refresh explicitly.';

	/// en-US: 'No supported native authorization path is available yet.'
	String get mobileNoSupportedNativeAuthorizationPathIsAvailableYet => 'No supported native authorization path is available yet.';

	/// en-US: 'None'
	String get mobileNone => 'None';

	/// en-US: 'Not available'
	String get mobileNotAvailable => 'Not available';

	/// en-US: 'Not available yet'
	String get mobileNotAvailableYet => 'Not available yet';

	/// en-US: 'Not listed \u00b7 live requests blocked'
	String get mobileNotListedLiveRequestsBlocked => 'Not listed \u00b7 live requests blocked';

	/// en-US: 'Not provided by platform'
	String get mobileNotProvidedByPlatform => 'Not provided by platform';

	/// en-US: 'Not running'
	String get mobileNotRunning => 'Not running';

	/// en-US: 'Observed readings'
	String get mobileObservedReadings => 'Observed readings';

	/// en-US: 'One appliance, separate components. Each uses its own supported limits.'
	String get mobileOneApplianceSeparateComponentsEachUsesItsOwn => 'One appliance, separate components. Each uses its own supported limits.';

	/// en-US: 'Only mainland China is supported. Browser OAuth, code challenges and appliance approval are unavailable.'
	String get mobileOnlyMainlandChinaIsSupportedBrowserOauthCode => 'Only mainland China is supported. Browser OAuth, code challenges and appliance approval are unavailable.';

	/// en-US: 'Only routes supported by this device are available.'
	String get mobileOnlyRoutesSupportedByThisDeviceAreAvailable => 'Only routes supported by this device are available.';

	/// en-US: 'Open Settings'
	String get mobileOpenSettings => 'Open Settings';

	/// en-US: 'Open System Settings'
	String get mobileOpenSystemSettings => 'Open System Settings';

	/// en-US: 'Open an item for its evidence. Completed actions are not rolled back.'
	String get mobileOpenAnItemForItsEvidenceCompletedActions => 'Open an item for its evidence. Completed actions are not rolled back.';

	/// en-US: 'Opening this page only reads local account metadata. Saved authorization does not prove current access.'
	String get mobileOpeningThisPageOnlyReadsLocalAccountMetadata => 'Opening this page only reads local account metadata. Saved authorization does not prove current access.';

	/// en-US: 'Operation details'
	String get mobileOperationDetails => 'Operation details';

	/// en-US: 'Operation receipts are retained.'
	String get mobileOperationReceiptsAreRetained => 'Operation receipts are retained.';

	/// en-US: 'Operation results'
	String get mobileOperationResults => 'Operation results';

	/// en-US: 'Operation was not completed.'
	String get mobileOperationWasNotCompleted => 'Operation was not completed.';

	/// en-US: 'Options come from this device\u2019s capabilities.'
	String get mobileOptionsComeFromThisDeviceSCapabilities => 'Options come from this device\u2019s capabilities.';

	/// en-US: 'Other installations and provider authorization are unchanged.'
	String get mobileOtherInstallationsAndProviderAuthorizationAreUnchanged => 'Other installations and provider authorization are unchanged.';

	/// en-US: 'Outcome'
	String get mobileOutcome => 'Outcome';

	/// en-US: 'Paused'
	String get mobilePaused => 'Paused';

	/// en-US: 'Permission'
	String get mobilePermission => 'Permission';

	/// en-US: 'Permission change cancelled. Reopen read-only.'
	String get mobilePermissionChangeCancelledReopenReadOnly => 'Permission change cancelled. Reopen read-only.';

	/// en-US: 'Permissions'
	String get mobilePermissions => 'Permissions';

	/// en-US: 'Phase'
	String get mobilePhase => 'Phase';

	/// en-US: 'Planned'
	String get mobilePlanned => 'Planned';

	/// en-US: 'Platform account settings'
	String get mobilePlatformAccountSettings => 'Platform account settings';

	/// en-US: 'Platform authorization is unavailable in Mock. This profile contains fictional devices only.'
	String get mobilePlatformAuthorizationIsUnavailableInMockThisProfile => 'Platform authorization is unavailable in Mock. This profile contains fictional devices only.';

	/// en-US: 'Possible duplicate'
	String get mobilePossibleDuplicate => 'Possible duplicate';

	/// en-US: 'Possibly sent'
	String get mobilePossiblySent => 'Possibly sent';

	/// en-US: 'Power unknown'
	String get mobilePowerUnknown => 'Power unknown';

	/// en-US: 'Preset'
	String get mobilePreset => 'Preset';

	/// en-US: 'Preview'
	String get mobilePreview => 'Preview';

	/// en-US: 'Preview Skin'
	String get mobilePreviewSkin => 'Preview Skin';

	/// en-US: 'Preview only'
	String get mobilePreviewOnly => 'Preview only';

	/// en-US: 'Preview only: no command was sent.'
	String get mobilePreviewOnlyNoCommandWasSent => 'Preview only: no command was sent.';

	/// en-US: 'Preview only: no device setting was changed.'
	String get mobilePreviewOnlyNoDeviceSettingWasChanged => 'Preview only: no device setting was changed.';

	/// en-US: 'Preview only: no device was contacted.'
	String get mobilePreviewOnlyNoDeviceWasContacted => 'Preview only: no device was contacted.';

	/// en-US: 'Previous read'
	String get mobilePreviousRead => 'Previous read';

	/// en-US: 'Protected account connection requires native API 1.2 or later.'
	String get mobileProtectedAccountConnectionRequiresNativeApi12 => 'Protected account connection requires native API 1.2 or later.';

	/// en-US: 'Provider revocation unavailable'
	String get mobileProviderRevocationUnavailable => 'Provider revocation unavailable';

	/// en-US: 'Read'
	String get mobileRead => 'Read';

	/// en-US: 'Read a supported target first.'
	String get mobileReadASupportedTargetFirst => 'Read a supported target first.';

	/// en-US: 'Read and control'
	String get mobileReadAndControl => 'Read and control';

	/// en-US: 'Read device state'
	String get mobileReadDeviceState => 'Read device state';

	/// en-US: 'Read state before changing power.'
	String get mobileReadStateBeforeChangingPower => 'Read state before changing power.';

	/// en-US: 'Ready'
	String get mobileReady => 'Ready';

	/// en-US: 'Receipt / attempt limits'
	String get mobileReceiptAttemptLimits => 'Receipt / attempt limits';

	/// en-US: 'Recent searches'
	String get mobileRecentSearches => 'Recent searches';

	/// en-US: 'Recovery validates saved authorization and retrieves cloud devices. It preserves local organization and sends no appliance controls.'
	String get mobileRecoveryValidatesSavedAuthorizationAndRetrievesCloudDevices => 'Recovery validates saved authorization and retrieves cloud devices. It preserves local organization and sends no appliance controls.';

	/// en-US: 'Refresh state before using a quick power action.'
	String get mobileRefreshStateBeforeUsingAQuickPowerAction => 'Refresh state before using a quick power action.';

	/// en-US: 'Refrigerator'
	String get mobileRefrigerator => 'Refrigerator';

	/// en-US: 'Region'
	String get mobileRegion => 'Region';

	/// en-US: 'Regions and compatibility'
	String get mobileRegionsAndCompatibility => 'Regions and compatibility';

	/// en-US: 'Reload local data before confirming another cleanup.'
	String get mobileReloadLocalDataBeforeConfirmingAnotherCleanup => 'Reload local data before confirming another cleanup.';

	/// en-US: 'Remove From This Device'
	String get mobileRemoveFromThisDevice => 'Remove From This Device';

	/// en-US: 'Remove Local Data'
	String get mobileRemoveLocalData => 'Remove Local Data';

	/// en-US: 'Remove Locally'
	String get mobileRemoveLocally => 'Remove Locally';

	/// en-US: 'Remove detailed operation receipts, including uncertain outcomes and their evidence. Request IDs remain reserved: clearing history never permits replay. Active operations prevent cleanup.'
	String get mobileRemoveDetailedOperationReceiptsIncludingUncertainOutcomesAnd => 'Remove detailed operation receipts, including uncertain outcomes and their evidence. Request IDs remain reserved: clearing history never permits replay. Active operations prevent cleanup.';

	/// en-US: 'Remove from this device?'
	String get mobileRemoveFromThisDevice496642 => 'Remove from this device?';

	/// en-US: 'Remove local data'
	String get mobileRemoveLocalDataCb7713 => 'Remove local data';

	/// en-US: 'Remove local data?'
	String get mobileRemoveLocalData6afed9 => 'Remove local data?';

	/// en-US: 'Remove saved device and component readings from this installation. Missing readings become unknown. No device refresh will start.'
	String get mobileRemoveSavedDeviceAndComponentReadingsFromThis => 'Remove saved device and component readings from this installation. Missing readings become unknown. No device refresh will start.';

	/// en-US: 'Removed from this device'
	String get mobileRemovedFromThisDevice => 'Removed from this device';

	/// en-US: 'Rename Account'
	String get mobileRenameAccount => 'Rename Account';

	/// en-US: 'Rename Device'
	String get mobileRenameDevice => 'Rename Device';

	/// en-US: 'Reopen the application.'
	String get mobileReopenTheApplication => 'Reopen the application.';

	/// en-US: 'Reorder devices'
	String get mobileReorderDevices0c0797 => 'Reorder devices';

	/// en-US: 'Requested'
	String get mobileRequested => 'Requested';

	/// en-US: 'Requested access'
	String get mobileRequestedAccess => 'Requested access';

	/// en-US: 'Result unknown'
	String get mobileResultUnknown => 'Result unknown';

	/// en-US: 'Retained device'
	String get mobileRetainedDevice => 'Retained device';

	/// en-US: 'Retrieve Devices explicitly to check membership.'
	String get mobileRetrieveDevicesExplicitlyToCheckMembership => 'Retrieve Devices explicitly to check membership.';

	/// en-US: 'Return to the foreground to reopen read-only.'
	String get mobileReturnToTheForegroundToReopenReadOnly => 'Return to the foreground to reopen read-only.';

	/// en-US: 'Return to the foreground.'
	String get mobileReturnToTheForeground => 'Return to the foreground.';

	/// en-US: 'Review Full Report'
	String get mobileReviewFullReport => 'Review Full Report';

	/// en-US: 'Review a new confirmation.'
	String get mobileReviewANewConfirmation => 'Review a new confirmation.';

	/// en-US: 'Review before saving. Account and device identifiers are excluded.'
	String get mobileReviewBeforeSavingAccountAndDeviceIdentifiersAre => 'Review before saving. Account and device identifiers are excluded.';

	/// en-US: 'Review permissions for this device first.'
	String get mobileReviewPermissionsForThisDeviceFirst => 'Review permissions for this device first.';

	/// en-US: 'Review removal scope'
	String get mobileReviewRemovalScope => 'Review removal scope';

	/// en-US: 'Review the command again before confirming.'
	String get mobileReviewTheCommandAgainBeforeConfirming => 'Review the command again before confirming.';

	/// en-US: 'Review the local removal scope first.'
	String get mobileReviewTheLocalRemovalScopeFirst => 'Review the local removal scope first.';

	/// en-US: 'Review the scope'
	String get mobileReviewTheScope => 'Review the scope';

	/// en-US: 'Review this device again before allowing controls.'
	String get mobileReviewThisDeviceAgainBeforeAllowingControls => 'Review this device again before allowing controls.';

	/// en-US: 'Revocation'
	String get mobileRevocation => 'Revocation';

	/// en-US: 'Room membership stays unchanged. Hidden devices retain their position.'
	String get mobileRoomMembershipStaysUnchangedHiddenDevicesRetainTheir => 'Room membership stays unchanged. Hidden devices retain their position.';

	/// en-US: 'Rooms'
	String get mobileRooms => 'Rooms';

	/// en-US: 'Rooms and names'
	String get mobileRoomsAndNames => 'Rooms and names';

	/// en-US: 'Rooms organize devices locally; access stays the same.'
	String get mobileRoomsOrganizeDevicesLocallyAccessStaysTheSame => 'Rooms organize devices locally; access stays the same.';

	/// en-US: 'Rooms, operation receipts and replay protection remain. Other installations and provider authorization are unaffected. Reconnecting creates new local identities without restoring previous organization.'
	String get mobileRoomsOperationReceiptsAndReplayProtectionRemainOther => 'Rooms, operation receipts and replay protection remain. Other installations and provider authorization are unaffected. Reconnecting creates new local identities without restoring previous organization.';

	/// en-US: 'Running'
	String get mobileRunning => 'Running';

	/// en-US: 'Save Mode'
	String get mobileSaveMode => 'Save Mode';

	/// en-US: 'Save Preference'
	String get mobileSavePreference => 'Save Preference';

	/// en-US: 'Save Room'
	String get mobileSaveRoom => 'Save Room';

	/// en-US: 'Save target'
	String get mobileSaveTarget => 'Save target';

	/// en-US: 'Saved authorization \u00b7 not checked'
	String get mobileSavedAuthorizationNotChecked => 'Saved authorization \u00b7 not checked';

	/// en-US: 'Saved binding without a matching card'
	String get mobileSavedBindingWithoutAMatchingCard => 'Saved binding without a matching card';

	/// en-US: 'Saved connection'
	String get mobileSavedConnection => 'Saved connection';

	/// en-US: 'Saved device readings are retained.'
	String get mobileSavedDeviceReadingsAreRetained => 'Saved device readings are retained.';

	/// en-US: 'Saved inventory only. This does not check current access, online state, model support or control permission. Historical inventory time is shown separately when recorded.'
	String get mobileSavedInventoryOnlyThisDoesNotCheckCurrent => 'Saved inventory only. This does not check current access, online state, model support or control permission. Historical inventory time is shown separately when recorded.';

	/// en-US: 'Saved locally'
	String get mobileSavedLocally => 'Saved locally';

	/// en-US: 'Saved membership'
	String get mobileSavedMembership => 'Saved membership';

	/// en-US: 'Saved on this installation. Device access stays the same.'
	String get mobileSavedOnThisInstallationDeviceAccessStaysThe => 'Saved on this installation. Device access stays the same.';

	/// en-US: 'Saved records may refer to the same appliance. Physical identity is unverified. No devices are merged and no permission is transferred.'
	String get mobileSavedRecordsMayReferToTheSameAppliance => 'Saved records may refer to the same appliance. Physical identity is unverified. No devices are merged and no permission is transferred.';

	/// en-US: 'Saved records only'
	String get mobileSavedRecordsOnly => 'Saved records only';

	/// en-US: 'Saved states may be out of date.'
	String get mobileSavedStatesMayBeOutOfDate => 'Saved states may be out of date.';

	/// en-US: 'Scope'
	String get mobileScope => 'Scope';

	/// en-US: 'Search'
	String get mobileSearch => 'Search';

	/// en-US: 'Search Again'
	String get mobileSearchAgain => 'Search Again';

	/// en-US: 'Search platforms'
	String get mobileSearchPlatforms => 'Search platforms';

	/// en-US: 'Searches local device names and rooms.'
	String get mobileSearchesLocalDeviceNamesAndRooms => 'Searches local device names and rooms.';

	/// en-US: 'Secure storage unavailable'
	String get mobileSecureStorageUnavailable => 'Secure storage unavailable';

	/// en-US: 'Select 1\u201332 distinct devices.'
	String get mobileSelect132DistinctDevices => 'Select 1\u201332 distinct devices.';

	/// en-US: 'Select a device receipt to refresh.'
	String get mobileSelectADeviceReceiptToRefresh => 'Select a device receipt to refresh.';

	/// en-US: 'Select a record to inspect the evidence.'
	String get mobileSelectARecordToInspectTheEvidence => 'Select a record to inspect the evidence.';

	/// en-US: 'Select a saved account.'
	String get mobileSelectASavedAccount => 'Select a saved account.';

	/// en-US: 'Select a saved device connection.'
	String get mobileSelectASavedDeviceConnection => 'Select a saved device connection.';

	/// en-US: 'Select a saved inventory binding.'
	String get mobileSelectASavedInventoryBinding => 'Select a saved inventory binding.';

	/// en-US: 'Select a stored account with saved-inventory inspection support.'
	String get mobileSelectAStoredAccountWithSavedInventoryInspection => 'Select a stored account with saved-inventory inspection support.';

	/// en-US: 'Select an available component.'
	String get mobileSelectAnAvailableComponent => 'Select an available component.';

	/// en-US: 'Select an operation to inspect its retained evidence. Sharing history is unavailable.'
	String get mobileSelectAnOperationToInspectItsRetainedEvidence => 'Select an operation to inspect its retained evidence. Sharing history is unavailable.';

	/// en-US: 'Select at least one device to move.'
	String get mobileSelectAtLeastOneDeviceToMove => 'Select at least one device to move.';

	/// en-US: 'Select devices'
	String get mobileSelectDevices224608 => 'Select devices';

	/// en-US: 'Select devices and a destination room. Access stays unchanged.'
	String get mobileSelectDevicesAndADestinationRoomAccessStays => 'Select devices and a destination room. Access stays unchanged.';

	/// en-US: 'Select only reviewed values.'
	String get mobileSelectOnlyReviewedValues => 'Select only reviewed values.';

	/// en-US: 'Selected'
	String get mobileSelected => 'Selected';

	/// en-US: 'Selection requires one unambiguous low-risk power control per device.'
	String get mobileSelectionRequiresOneUnambiguousLowRiskPowerControl => 'Selection requires one unambiguous low-risk power control per device.';

	/// en-US: 'Set Temperature'
	String get mobileSetTemperature => 'Set Temperature';

	/// en-US: 'Sharing'
	String get mobileSharing => 'Sharing';

	/// en-US: 'Sign in to the original account. Identity must match; local device names, rooms and favorites are preserved. This does not grant new controls.'
	String get mobileSignInToTheOriginalAccountIdentityMust => 'Sign in to the original account. Identity must match; local device names, rooms and favorites are preserved. This does not grant new controls.';

	/// en-US: 'Sign-in succeeded.'
	String get mobileSignInSucceeded => 'Sign-in succeeded.';

	/// en-US: 'Sign-in succeeded. Saved authorization \u00b7 not checked.'
	String get mobileSignInSucceededSavedAuthorizationNotChecked => 'Sign-in succeeded. Saved authorization \u00b7 not checked.';

	/// en-US: 'Signing in to a pending account requires native API 1.8 or later.'
	String get mobileSigningInToAPendingAccountRequiresNative => 'Signing in to a pending account requires native API 1.8 or later.';

	/// en-US: 'Simulated events never call Core.'
	String get mobileSimulatedEventsNeverCallCore => 'Simulated events never call Core.';

	/// en-US: 'Skin'
	String get mobileSkin => 'Skin';

	/// en-US: 'Smart Appliance Platforms'
	String get mobileSmartAppliancePlatforms => 'Smart Appliance Platforms';

	/// en-US: 'Soft window light, porcelain surfaces and warm terracotta controls.'
	String get mobileSoftWindowLightPorcelainSurfacesAndWarmTerracotta => 'Soft window light, porcelain surfaces and warm terracotta controls.';

	/// en-US: 'Some cleanup actions require a native library update.'
	String get mobileSomeCleanupActionsRequireANativeLibraryUpdate => 'Some cleanup actions require a native library update.';

	/// en-US: 'Start Again'
	String get mobileStartAgain => 'Start Again';

	/// en-US: 'Start Cycle'
	String get mobileStartCycle => 'Start Cycle';

	/// en-US: 'State matches the request; this alone does not prove causality.'
	String get mobileStateMatchesTheRequestThisAloneDoesNot => 'State matches the request; this alone does not prove causality.';

	/// en-US: 'State matching alone does not prove causality.'
	String get mobileStateMatchingAloneDoesNotProveCausality => 'State matching alone does not prove causality.';

	/// en-US: 'State verified'
	String get mobileStateVerified => 'State verified';

	/// en-US: 'Support is specific to platform, region and model.'
	String get mobileSupportIsSpecificToPlatformRegionAndModel => 'Support is specific to platform, region and model.';

	/// en-US: 'Support varies by platform, model and region.'
	String get mobileSupportVariesByPlatformModelAndRegion => 'Support varies by platform, model and region.';

	/// en-US: 'Supported appliances'
	String get mobileSupportedAppliances => 'Supported appliances';

	/// en-US: 'Supported step: 1\u00b0C for this fictional device.'
	String get mobileSupportedStep1CForThisFictionalDevice => 'Supported step: 1\u00b0C for this fictional device.';

	/// en-US: 'Target controls are unavailable.'
	String get mobileTargetControlsAreUnavailable => 'Target controls are unavailable.';

	/// en-US: 'Temperature'
	String get mobileTemperature => 'Temperature';

	/// en-US: 'Temperature is read-only here.'
	String get mobileTemperatureIsReadOnlyHere => 'Temperature is read-only here.';

	/// en-US: 'Text follows system accessibility settings.'
	String get mobileTextFollowsSystemAccessibilitySettings => 'Text follows system accessibility settings.';

	/// en-US: 'Text size and accessibility'
	String get mobileTextSizeAndAccessibility => 'Text size and accessibility';

	/// en-US: 'The account is no longer saved.'
	String get mobileTheAccountIsNoLongerSaved => 'The account is no longer saved.';

	/// en-US: 'The appliance continues if you close HomeVerb.'
	String get mobileTheApplianceContinuesIfYouCloseHomeverb => 'The appliance continues if you close HomeVerb.';

	/// en-US: 'The confirmation was cancelled.'
	String get mobileTheConfirmationWasCancelled => 'The confirmation was cancelled.';

	/// en-US: 'The list changed. Review the refreshed order and try again.'
	String get mobileTheListChangedReviewTheRefreshedOrderAnd => 'The list changed. Review the refreshed order and try again.';

	/// en-US: 'The pending request has not replaced the last observation.'
	String get mobileThePendingRequestHasNotReplacedTheLast => 'The pending request has not replaced the last observation.';

	/// en-US: 'This Device'
	String get mobileThisDevice => 'This Device';

	/// en-US: 'This account flow is unavailable.'
	String get mobileThisAccountFlowIsUnavailable => 'This account flow is unavailable.';

	/// en-US: 'This action has no trusted confirmation path.'
	String get mobileThisActionHasNoTrustedConfirmationPath => 'This action has no trusted confirmation path.';

	/// en-US: 'This action is not available.'
	String get mobileThisActionIsNotAvailable => 'This action is not available.';

	/// en-US: 'This authorization or approval method is not supported by the current native adapter.'
	String get mobileThisAuthorizationOrApprovalMethodIsNotSupported => 'This authorization or approval method is not supported by the current native adapter.';

	/// en-US: 'This device'
	String get mobileThisDeviceD05257 => 'This device';

	/// en-US: 'This device is read-only. Missing readings are not zero.'
	String get mobileThisDeviceIsReadOnlyMissingReadingsAre => 'This device is read-only. Missing readings are not zero.';

	/// en-US: 'This device was retrieved from the platform, but this model has no admitted readings or controls.'
	String get mobileThisDeviceWasRetrievedFromThePlatformBut => 'This device was retrieved from the platform, but this model has no admitted readings or controls.';

	/// en-US: 'This does not securely erase backups or storage pages.'
	String get mobileThisDoesNotSecurelyEraseBackupsOrStorage => 'This does not securely erase backups or storage pages.';

	/// en-US: 'This host cannot configure real controls.'
	String get mobileThisHostCannotConfigureRealControls => 'This host cannot configure real controls.';

	/// en-US: 'This operation is unavailable in the current native session.'
	String get mobileThisOperationIsUnavailableInTheCurrentNative => 'This operation is unavailable in the current native session.';

	/// en-US: 'This profile'
	String get mobileThisProfile => 'This profile';

	/// en-US: 'Too many exact session permissions.'
	String get mobileTooManyExactSessionPermissions => 'Too many exact session permissions.';

	/// en-US: 'Trusted approval is not available in this host.'
	String get mobileTrustedApprovalIsNotAvailableInThisHost => 'Trusted approval is not available in this host.';

	/// en-US: 'Trusted approval requires API 1.13 or later.'
	String get mobileTrustedApprovalRequiresApi113OrLater => 'Trusted approval requires API 1.13 or later.';

	/// en-US: 'Turn off'
	String get mobileTurnOff => 'Turn off';

	/// en-US: 'Turn on'
	String get mobileTurnOn => 'Turn on';

	/// en-US: 'Unconfirmed'
	String get mobileUnconfirmed => 'Unconfirmed';

	/// en-US: 'Unknown membership'
	String get mobileUnknownMembership => 'Unknown membership';

	/// en-US: 'Unrecognized account result. Reload local accounts.'
	String get mobileUnrecognizedAccountResultReloadLocalAccounts => 'Unrecognized account result. Reload local accounts.';

	/// en-US: 'Unrecognized removal result. Reload local accounts.'
	String get mobileUnrecognizedRemovalResultReloadLocalAccounts => 'Unrecognized removal result. Reload local accounts.';

	/// en-US: 'Unsupported account state'
	String get mobileUnsupportedAccountState => 'Unsupported account state';

	/// en-US: 'Updated'
	String get mobileUpdated => 'Updated';

	/// en-US: 'Updated just now'
	String get mobileUpdatedJustNow => 'Updated just now';

	/// en-US: 'Updated just now \u00b7 Limits come from this device.'
	String get mobileUpdatedJustNowLimitsComeFromThisDevice => 'Updated just now \u00b7 Limits come from this device.';

	/// en-US: 'Use 1\u2013256 ASCII bytes for each credential.'
	String get mobileUse1256AsciiBytesForEachCredential => 'Use 1\u2013256 ASCII bytes for each credential.';

	/// en-US: 'Use 1\u2013256 ASCII characters in each field; the account cannot contain spaces.'
	String get mobileUse1256AsciiCharactersInEachField => 'Use 1\u2013256 ASCII characters in each field; the account cannot contain spaces.';

	/// en-US: 'Use 1\u201380 characters.'
	String get mobileUse180Characters => 'Use 1\u201380 characters.';

	/// en-US: 'Use a whole number.'
	String get mobileUseAWholeNumber => 'Use a whole number.';

	/// en-US: 'Use whole degrees for this device.'
	String get mobileUseWholeDegreesForThisDevice => 'Use whole degrees for this device.';

	/// en-US: 'Use your existing Meiju account. After sign-in, saved authorization is used to retrieve bound devices automatically. No appliance controls are granted.'
	String get mobileUseYourExistingMeijuAccountAfterSignIn => 'Use your existing Meiju account. After sign-in, saved authorization is used to retrieve bound devices automatically. No appliance controls are granted.';

	/// en-US: 'Validated'
	String get mobileValidated => 'Validated';

	/// en-US: 'Verification stopped'
	String get mobileVerificationStopped => 'Verification stopped';

	/// en-US: 'Version'
	String get mobileVersion => 'Version';

	/// en-US: 'View Details'
	String get mobileViewDetails => 'View Details';

	/// en-US: 'View Devices'
	String get mobileViewDevices => 'View Devices';

	/// en-US: 'View Evidence'
	String get mobileViewEvidence => 'View Evidence';

	/// en-US: 'View Local Record'
	String get mobileViewLocalRecord => 'View Local Record';

	/// en-US: 'Wait for active operations.'
	String get mobileWaitForActiveOperations => 'Wait for active operations.';

	/// en-US: 'Waiting'
	String get mobileWaiting => 'Waiting';

	/// en-US: 'Waiting for an updated reading'
	String get mobileWaitingForAnUpdatedReading => 'Waiting for an updated reading';

	/// en-US: 'Window Light is the only included skin and is already selected.'
	String get mobileWindowLightIsTheOnlyIncludedSkinAnd => 'Window Light is the only included skin and is already selected.';

	/// en-US: 'Window Light \u00b7 Included locally'
	String get mobileWindowLightIncludedLocally => 'Window Light \u00b7 Included locally';

	/// en-US: 'Your home, together'
	String get mobileYourHomeTogether => 'Your home, together';

	/// en-US: 'Your platform authorization has expired.'
	String get mobileYourPlatformAuthorizationHasExpired => 'Your platform authorization has expired.';

	/// en-US: 'Your saved authorization will not be overwritten automatically.'
	String get mobileYourSavedAuthorizationWillNotBeOverwrittenAutomatically => 'Your saved authorization will not be overwritten automatically.';

	/// en-US: 'detailed receipts'
	String get mobileDetailedReceipts => 'detailed receipts';

	/// en-US: 'saved snapshots'
	String get mobileSavedSnapshots => 'saved snapshots';

	/// en-US: 'the selected account'
	String get mobileTheSelectedAccount => 'the selected account';

	/// en-US: '\u2014 Not available'
	String get mobileNotAvailable75d339 => '\u2014  Not available';

	/// en-US: 'Supported: {capabilities}'
	String mobileSupportedCapabilities({required Object capabilities}) => 'Supported: ${capabilities}';

	/// en-US: '{transport} connection'
	String mobileTransportConnection({required Object transport}) => '${transport} connection';

	/// en-US: 'Connection: {connection}'
	String mobileConnectionConnection({required Object connection}) => 'Connection: ${connection}';

	/// en-US: 'Available routes: {routes}. Core validates every selected route.'
	String mobileAvailableRoutesRoutesCoreValidatesEverySelectedRoute({required Object routes}) => 'Available routes: ${routes}. Core validates every selected route.';

	/// en-US: 'Delete {room}?'
	String mobileDeleteRoom8acbc3({required Object room}) => 'Delete ${room}?';

	/// en-US: '{source} observation'
	String mobileSourceObservation({required Object source}) => '${source} observation';

	/// en-US: '{mode} \u203a'
	String mobileMode({required Object mode}) => '${mode}  \u203a';

	/// en-US: 'Observed target remains {value}.'
	String mobileObservedTargetRemainsValue({required Object value}) => 'Observed target remains ${value}.';

	/// en-US: '{count} {kind} removed. New operations can add data again.'
	String mobileCountKindRemovedNewOperationsCanAddData({required Object count, required Object kind}) => '${count} ${kind} removed. New operations can add data again.';

	/// en-US: 'Request: {request}'
	String mobileRequestRequest({required Object request}) => 'Request: ${request}';

	/// en-US: '{room} \u00b7 Drag handles change the visible order.'
	String mobileRoomDragHandlesChangeTheVisibleOrder({required Object room}) => '${room} \u00b7 Drag handles change the visible order.';

	/// en-US: '{outcome} \u00b7 {time}'
	String mobileOutcomeTime({required Object outcome, required Object time}) => '${outcome} \u00b7 ${time}';

	/// en-US: '{count} devices retrieved. Unsupported models remain visible. An empty result does not delete older saved cards.'
	String mobileCountDevicesRetrievedUnsupportedModelsRemainVisibleAn({required Object count}) => '${count} devices retrieved. Unsupported models remain visible. An empty result does not delete older saved cards.';

	/// en-US: 'Meiju \u00b7 {region}'
	String mobileMeijuRegion({required Object region}) => 'Meiju \u00b7 ${region}';

	/// en-US: '{count} saved cards'
	String mobileCountSavedCards({required Object count}) => '${count} saved cards';

	/// en-US: 'Last complete device list: {time}. Recorded using this device\u2019s clock; not current authorization or appliance status.'
	String mobileLastCompleteDeviceListTimeRecordedUsingThis({required Object time}) => 'Last complete device list: ${time}. Recorded using this device\u2019s clock; not current authorization or appliance status.';

	/// en-US: 'Selected device: {device}'
	String mobileSelectedDeviceDevice({required Object device}) => 'Selected device: ${device}';

	/// en-US: 'Remove {account}, its local credentials and {count} linked cards, preferences and saved readings.'
	String mobileRemoveAccountItsLocalCredentialsAndCountLinked({required Object account, required Object count}) => 'Remove ${account}, its local credentials and ${count} linked cards, preferences and saved readings.';

	/// en-US: '{authentication} {storage}.'
	String mobileAuthenticationStorage({required Object authentication, required Object storage}) => '${authentication} ${storage}.';

	/// en-US: 'Choose {name}.'
	String mobileChooseName({required Object name}) => 'Choose ${name}.';

	/// en-US: 'Choose at least {count} value(s).'
	String mobileChooseAtLeastCountValueS({required Object count}) => 'Choose at least ${count} value(s).';

	/// en-US: '{name} \u00b7 {requirement}'
	String mobileNameRequirement({required Object name, required Object requirement}) => '${name} \u00b7 ${requirement}';

	/// en-US: 'Observed: {value}'
	String mobileObservedValue({required Object value}) => 'Observed: ${value}';

	/// en-US: 'Read: {time}{stale}'
	String mobileReadTimeStale({required Object time, required Object stale}) => 'Read: ${time}${stale}';

	/// en-US: 'Minimum {value}'
	String mobileMinimumValue({required Object value}) => 'Minimum ${value}';

	/// en-US: 'Maximum {value}'
	String mobileMaximumValue({required Object value}) => 'Maximum ${value}';

	/// en-US: 'Step {value}'
	String mobileStepValue({required Object value}) => 'Step ${value}';

	/// en-US: 'Target temperature: {value}\u00b0C'
	String mobileTargetTemperatureValueC({required Object value}) => 'Target temperature: ${value}\u00b0C';

	/// en-US: 'Turn on and select mode: {mode}'
	String mobileTurnOnAndSelectModeMode({required Object mode}) => 'Turn on and select mode: ${mode}';

	/// en-US: 'Fan speed: {value}'
	String mobileFanSpeedValue({required Object value}) => 'Fan speed: ${value}';

	/// en-US: 'Attempt {count}'
	String mobileAttemptCount({required Object count}) => 'Attempt ${count}';

	/// en-US: 'Step {count}'
	String mobileStepCount({required Object count}) => 'Step ${count}';

	/// en-US: 'Warning: {warning}'
	String mobileWarningWarning({required Object warning}) => 'Warning: ${warning}';

	/// en-US: 'Device: {device}'
	String mobileDeviceDevice({required Object device}) => 'Device: ${device}';

	/// en-US: 'Local ID: {value}'
	String mobileLocalIdValue({required Object value}) => 'Local ID: ${value}';

	/// en-US: 'Component: {value}'
	String mobileComponentValue({required Object value}) => 'Component: ${value}';

	/// en-US: 'Action: {value}'
	String mobileActionValue({required Object value}) => 'Action: ${value}';

	/// en-US: 'Arguments: {value}'
	String mobileArgumentsValue({required Object value}) => 'Arguments: ${value}';

	/// en-US: 'Expires: {value}'
	String mobileExpiresValue({required Object value}) => 'Expires: ${value}';

	/// en-US: 'Request: {value}'
	String mobileRequestValue({required Object value}) => 'Request: ${value}';

	/// en-US: 'Connection: {transport} \u00b7 {connection}'
	String mobileConnectionTransportConnection({required Object transport, required Object connection}) => 'Connection: ${transport} \u00b7 ${connection}';

	/// en-US: 'Required'
	String get mobileRequired => 'Required';

	/// en-US: 'Optional'
	String get mobileOptional => 'Optional';

	/// en-US: ' \u00b7 Stale'
	String get mobileStale412c56 => ' \u00b7 Stale';

	/// en-US: 'Room actions {name}'
	String mobileRoomActionsName({required Object name}) => 'Room actions ${name}';

	/// en-US: 'Reorder {name}'
	String mobileReorderName({required Object name}) => 'Reorder ${name}';

	/// en-US: 'Power {name}'
	String mobilePowerName({required Object name}) => 'Power ${name}';

	/// en-US: 'Open {name}'
	String mobileOpenName({required Object name}) => 'Open ${name}';

	/// en-US: 'Menu {name}'
	String mobileMenuName({required Object name}) => 'Menu ${name}';

	/// en-US: 'Dismiss {name}'
	String mobileDismissName({required Object name}) => 'Dismiss ${name}';

	/// en-US: 'Increase {name}'
	String mobileIncreaseName({required Object name}) => 'Increase ${name}';

	/// en-US: 'Decrease {name}'
	String mobileDecreaseName({required Object name}) => 'Decrease ${name}';

	/// en-US: 'Enter {name}'
	String mobileEnterName({required Object name}) => 'Enter ${name}';

	/// en-US: '{route} \u00b7 {phase} \u00b7 {duration} ms'
	String mobileRoutePhaseDurationMs({required Object route, required Object phase, required Object duration}) => '${route} \u00b7 ${phase} \u00b7 ${duration} ms';

	/// en-US: '{state}{suffix}'
	String mobileStateSuffix({required Object state, required Object suffix}) => '${state}${suffix}';

	/// en-US: 'Saved card \u00b7 {suffix}'
	String mobileSavedCardSuffix({required Object suffix}) => 'Saved card \u00b7 ${suffix}';

	/// en-US: '{count} device'
	String mobileCountDevice({required Object count}) => '${count} device';

	/// en-US: '{count} devices'
	String mobileCountDevices({required Object count}) => '${count} devices';

	/// en-US: '{value} \u00b7 Stale'
	String mobileValueStale({required Object value}) => '${value} \u00b7 Stale';

	/// en-US: 'Latest read: {time}'
	String mobileLatestReadTime({required Object time}) => 'Latest read: ${time}';

	/// en-US: 'Set {name}'
	String mobileSetName({required Object name}) => 'Set ${name}';

	/// en-US: '{component} target'
	String mobileComponentTarget({required Object component}) => '${component} target';

	/// en-US: '{component} could not be read: {error}'
	String mobileComponentCouldNotBeReadError({required Object component, required Object error}) => '${component} could not be read: ${error}';

	/// en-US: '{component} \u00b7 No saved reading'
	String mobileComponentNoSavedReading({required Object component}) => '${component} \u00b7 No saved reading';

	/// en-US: '{component} \u00b7 {source} {time}{stale}'
	String mobileComponentSourceTimeStale({required Object component, required Object source, required Object time, required Object stale}) => '${component} \u00b7 ${source} ${time}${stale}';

	/// en-US: '{component} target is read-only.'
	String mobileComponentTargetIsReadOnly({required Object component}) => '${component} target is read-only.';

	/// en-US: '{label}: {message}'
	String mobileLabelMessage({required Object label, required Object message}) => '${label}: ${message}';

	/// en-US: 'Read observation'
	String get mobileReadObservation => 'Read observation';

	/// en-US: 'Main'
	String get mobileMain => 'Main';

	/// en-US: 'Wind speed'
	String get mobileWindSpeed => 'Wind speed';

	/// en-US: 'Activate mode'
	String get mobileActivateMode => 'Activate mode';

	/// en-US: 'Vendor.midea.ac'
	String get mobileVendorMideaAc => 'Vendor.midea.ac';

	/// en-US: 'Vendor.midea.dishwasher'
	String get mobileVendorMideaDishwasher => 'Vendor.midea.dishwasher';

	/// en-US: 'Work status'
	String get mobileWorkStatus => 'Work status';

	/// en-US: 'Additional'
	String get mobileAdditional => 'Additional';

	/// en-US: 'Remaining'
	String get mobileRemaining => 'Remaining';

	/// en-US: 'Remaining minutes'
	String get mobileRemainingMinutes => 'Remaining minutes';

	/// en-US: 'Current humidity'
	String get mobileCurrentHumidity => 'Current humidity';

	/// en-US: 'Temperature step'
	String get mobileTemperatureStep => 'Temperature step';

	/// en-US: 'Read-only scope'
	String get mobileReadOnlyScope => 'Read-only scope';

	/// en-US: '0 devices \u2261'
	String get mobile0Devices => '0 devices  \u2261';

	/// en-US: '0.2.0 \u00b7 Illustrative'
	String get mobile020Illustrative => '0.2.0 \u00b7 Illustrative';

	/// en-US: '01:20 remaining'
	String get mobile0120Remaining => '01:20 remaining';

	/// en-US: '1 device'
	String get mobile1Device => '1 device';

	/// en-US: '1 device \u2261'
	String get mobile1DeviceF72b66 => '1 device  \u2261';

	/// en-US: '1 device \u00b7 Checked yesterday'
	String get mobile1DeviceCheckedYesterday => '1 device \u00b7 Checked yesterday';

	/// en-US: '2 devices'
	String get mobile2Devices => '2 devices';

	/// en-US: '2 devices \u2261'
	String get mobile2DevicesF65433 => '2 devices  \u2261';

	/// en-US: '2 devices added'
	String get mobile2DevicesAdded => '2 devices added';

	/// en-US: '2 devices retrieved. Unsupported models remain visible. An empty result does not delete older saved cards.'
	String get mobile2DevicesRetrievedUnsupportedModelsRemainVisibleAn => '2 devices retrieved. Unsupported models remain visible. An empty result does not delete older saved cards.';

	/// en-US: '2 devices \u00b7 Sign-in needed'
	String get mobile2DevicesSignInNeeded => '2 devices \u00b7 Sign-in needed';

	/// en-US: '2 more devices below'
	String get mobile2MoreDevicesBelow => '2 more devices below';

	/// en-US: '3 devices \u00b7 Outcomes remain separate.'
	String get mobile3DevicesOutcomesRemainSeparate => '3 devices \u00b7 Outcomes remain separate.';

	/// en-US: '4 devices'
	String get mobile4Devices => '4 devices';

	/// en-US: '4 devices \u00b7 Updated just now'
	String get mobile4DevicesUpdatedJustNow => '4 devices \u00b7 Updated just now';

	/// en-US: '46% humidity'
	String get mobile46Humidity => '46% humidity';

	/// en-US: '58 min'
	String get mobile58Min => '58 min';

	/// en-US: '6 devices \u00b7 Checked today'
	String get mobile6DevicesCheckedToday => '6 devices \u00b7 Checked today';

	/// en-US: '6 seconds remaining'
	String get mobile6SecondsRemaining => '6 seconds remaining';

	/// en-US: '6 \u00b7 Last known'
	String get mobile6LastKnown => '6 \u00b7 Last known';

	/// en-US: 'AC \u00b7 TV'
	String get mobileAcTv => 'AC \u00b7 TV';

	/// en-US: 'Accept Authorization'
	String get mobileAcceptAuthorization => 'Accept Authorization';

	/// en-US: 'Accepted \u00b7 Awaiting state'
	String get mobileAcceptedAwaitingState => 'Accepted \u00b7 Awaiting state';

	/// en-US: 'Add 2 Devices'
	String get mobileAdd2Devices => 'Add 2 Devices';

	/// en-US: 'Air sensor'
	String get mobileAirSensor => 'Air sensor';

	/// en-US: 'Alex phone'
	String get mobileAlexPhone => 'Alex phone';

	/// en-US: 'Alex phone \u00b7 Visible for 01:48'
	String get mobileAlexPhoneVisibleFor0148 => 'Alex phone \u00b7 Visible for 01:48';

	/// en-US: 'Allow HomeVerb access?'
	String get mobileAllowHomeverbAccess => 'Allow HomeVerb access?';

	/// en-US: 'Ask the recipient to open Receive Authorization.'
	String get mobileAskTheRecipientToOpenReceiveAuthorization => 'Ask the recipient to open Receive Authorization.';

	/// en-US: 'Authorization blocked'
	String get mobileAuthorizationBlocked => 'Authorization blocked';

	/// en-US: 'Authorization is tied to the original device. The recipient must open Platforms and pair on their own device.'
	String get mobileAuthorizationIsTiedToTheOriginalDeviceThe => 'Authorization is tied to the original device. The recipient must open Platforms and pair on their own device.';

	/// en-US: 'Authorization received'
	String get mobileAuthorizationReceived => 'Authorization received';

	/// en-US: 'Authorization validated'
	String get mobileAuthorizationValidated => 'Authorization validated';

	/// en-US: 'Authorization was not imported. Enable secure storage before trying again.'
	String get mobileAuthorizationWasNotImportedEnableSecureStorageBefore => 'Authorization was not imported. Enable secure storage before trying again.';

	/// en-US: 'Auto \u203a'
	String get mobileAuto => 'Auto  \u203a';

	/// en-US: 'Balcony'
	String get mobileBalcony => 'Balcony';

	/// en-US: 'Bathroom'
	String get mobileBathroom => 'Bathroom';

	/// en-US: 'Bedroom'
	String get mobileBedroom => 'Bedroom';

	/// en-US: 'Bedroom AC'
	String get mobileBedroomAc => 'Bedroom AC';

	/// en-US: 'Bedroom AC \u00b7 Possible duplicate'
	String get mobileBedroomAcPossibleDuplicate => 'Bedroom AC \u00b7 Possible duplicate';

	/// en-US: 'Bedroom fan'
	String get mobileBedroomFan => 'Bedroom fan';

	/// en-US: 'Browser authorization \u203a'
	String get mobileBrowserAuthorization => 'Browser authorization  \u203a';

	/// en-US: 'Cached \u00b7 2h ago'
	String get mobileCached2hAgo => 'Cached \u00b7 2h ago';

	/// en-US: 'Cancel Transfer'
	String get mobileCancelTransfer => 'Cancel Transfer';

	/// en-US: 'Check'
	String get mobileCheck => 'Check';

	/// en-US: 'Check Connection'
	String get mobileCheckConnection => 'Check Connection';

	/// en-US: 'Choose a connected account \u203a'
	String get mobileChooseAConnectedAccount => 'Choose a connected account  \u203a';

	/// en-US: 'Codes Do Not Match'
	String get mobileCodesDoNotMatch => 'Codes Do Not Match';

	/// en-US: 'Codes Match'
	String get mobileCodesMatch => 'Codes Match';

	/// en-US: 'Codes did not match'
	String get mobileCodesDidNotMatch => 'Codes did not match';

	/// en-US: 'Compare this code on both devices. No authorization has been sent.'
	String get mobileCompareThisCodeOnBothDevicesNoAuthorization => 'Compare this code on both devices. No authorization has been sent.';

	/// en-US: 'Compare this code with the sender.'
	String get mobileCompareThisCodeWithTheSender => 'Compare this code with the sender.';

	/// en-US: 'Confirm Same Device'
	String get mobileConfirmSameDevice => 'Confirm Same Device';

	/// en-US: 'Confirm this fictional code on your TV.'
	String get mobileConfirmThisFictionalCodeOnYourTv => 'Confirm this fictional code on your TV.';

	/// en-US: 'Connect Example Home'
	String get mobileConnectExampleHome => 'Connect Example Home';

	/// en-US: 'Continue to Authorization'
	String get mobileContinueToAuthorization => 'Continue to Authorization';

	/// en-US: 'Control supported devices'
	String get mobileControlSupportedDevices => 'Control supported devices';

	/// en-US: 'Cool \u203a'
	String get mobileCool => 'Cool  \u203a';

	/// en-US: 'Cooling \u00b7 Room 26\u00b0C'
	String get mobileCoolingRoom26C => 'Cooling \u00b7 Room 26\u00b0C';

	/// en-US: 'Cotton \u203a'
	String get mobileCotton => 'Cotton  \u203a';

	/// en-US: 'Cotton \u00b7 Updated 09:42'
	String get mobileCottonUpdated0942 => 'Cotton \u00b7 Updated 09:42';

	/// en-US: 'Decline'
	String get mobileDecline => 'Decline';

	/// en-US: 'Delete Study?'
	String get mobileDeleteStudy => 'Delete Study?';

	/// en-US: 'Demo AC'
	String get mobileDemoAc => 'Demo AC';

	/// en-US: 'Demo Light'
	String get mobileDemoLight => 'Demo Light';

	/// en-US: 'Device is ready to pair'
	String get mobileDeviceIsReadyToPair => 'Device is ready to pair';

	/// en-US: 'Device pairing \u203a'
	String get mobileDevicePairing => 'Device pairing  \u203a';

	/// en-US: 'Discovered connection'
	String get mobileDiscoveredConnection => 'Discovered connection';

	/// en-US: 'Enable local network access in system settings to discover nearby devices.'
	String get mobileEnableLocalNetworkAccessInSystemSettingsTo => 'Enable local network access in system settings to discover nearby devices.';

	/// en-US: 'Encrypted transfer \u00b7 62%'
	String get mobileEncryptedTransfer62 => 'Encrypted transfer \u00b7 62%';

	/// en-US: 'Enter the code from your platform. Sent to ***42.'
	String get mobileEnterTheCodeFromYourPlatformSentTo => 'Enter the code from your platform. Sent to ***42.';

	/// en-US: 'Europe'
	String get mobileEurope => 'Europe';

	/// en-US: 'Europe \u203a'
	String get mobileEurope2fa2c4 => 'Europe  \u203a';

	/// en-US: 'Example Climate'
	String get mobileExampleClimate => 'Example Climate';

	/// en-US: 'Example Home'
	String get mobileExampleHome => 'Example Home';

	/// en-US: 'Example Home supports revoking this grant. Other installations using it may lose access.'
	String get mobileExampleHomeSupportsRevokingThisGrantOtherInstallations => 'Example Home supports revoking this grant. Other installations using it may lose access.';

	/// en-US: 'Example TV'
	String get mobileExampleTv => 'Example TV';

	/// en-US: 'Example TV \u00b7 Device-bound'
	String get mobileExampleTvDeviceBound => 'Example TV \u00b7 Device-bound';

	/// en-US: 'Example TV \u00b7 Living room'
	String get mobileExampleTvLivingRoom => 'Example TV \u00b7 Living room';

	/// en-US: 'Existing Bedroom AC'
	String get mobileExistingBedroomAc => 'Existing Bedroom AC';

	/// en-US: 'Existing authorization'
	String get mobileExistingAuthorization => 'Existing authorization';

	/// en-US: 'Failed \u00b7 Permission denied'
	String get mobileFailedPermissionDenied => 'Failed \u00b7 Permission denied';

	/// en-US: 'Fictional platform names; not support claims.'
	String get mobileFictionalPlatformNamesNotSupportClaims => 'Fictional platform names; not support claims.';

	/// en-US: 'Find Recipient'
	String get mobileFindRecipient => 'Find Recipient';

	/// en-US: 'Find a device'
	String get mobileFindADevice => 'Find a device';

	/// en-US: 'Find devices nearby'
	String get mobileFindDevicesNearby => 'Find devices nearby';

	/// en-US: 'Floor lamp'
	String get mobileFloorLamp => 'Floor lamp';

	/// en-US: 'Floor lamp \u203a'
	String get mobileFloorLamp4813bc => 'Floor lamp  \u203a';

	/// en-US: 'Floor lamp \u00b7 Local network'
	String get mobileFloorLampLocalNetwork => 'Floor lamp \u00b7 Local network';

	/// en-US: 'Floor lamp \u00b7 Set power On'
	String get mobileFloorLampSetPowerOn => 'Floor lamp \u00b7 Set power On';

	/// en-US: 'Found'
	String get mobileFound => 'Found';

	/// en-US: 'Found Bedroom AC'
	String get mobileFoundBedroomAc => 'Found Bedroom AC';

	/// en-US: 'Found devices'
	String get mobileFoundDevices => 'Found devices';

	/// en-US: 'From Home phone'
	String get mobileFromHomePhone => 'From Home phone';

	/// en-US: 'From a Platform'
	String get mobileFromAPlatform => 'From a Platform';

	/// en-US: 'Home account \u00b7 Example Home'
	String get mobileHomeAccountExampleHome => 'Home account \u00b7 Example Home';

	/// en-US: 'Home account \u00b7 Full account access'
	String get mobileHomeAccountFullAccountAccess => 'Home account \u00b7 Full account access';

	/// en-US: 'Home account \u00b7 h***@example.invalid'
	String get mobileHomeAccountHExampleInvalid => 'Home account \u00b7 h***@example.invalid';

	/// en-US: 'Home phone \u2192 Alex phone'
	String get mobileHomePhoneAlexPhone => 'Home phone \u2192 Alex phone';

	/// en-US: 'If sending had started, check the recipient before sending again.'
	String get mobileIfSendingHadStartedCheckTheRecipientBefore => 'If sending had started, check the recipient before sending again.';

	/// en-US: 'Illustrative accounts. Compatibility is model-specific.'
	String get mobileIllustrativeAccountsCompatibilityIsModelSpecific => 'Illustrative accounts. Compatibility is model-specific.';

	/// en-US: 'Import confirmation'
	String get mobileImportConfirmation => 'Import confirmation';

	/// en-US: 'Import requires system secure storage.'
	String get mobileImportRequiresSystemSecureStorage => 'Import requires system secure storage.';

	/// en-US: 'Imported authorization is not ready for use.'
	String get mobileImportedAuthorizationIsNotReadyForUse => 'Imported authorization is not ready for use.';

	/// en-US: 'Imported securely'
	String get mobileImportedSecurely => 'Imported securely';

	/// en-US: 'Imported \u00b7 09:38'
	String get mobileImported0938 => 'Imported \u00b7 09:38';

	/// en-US: 'Incoming authorization'
	String get mobileIncomingAuthorization => 'Incoming authorization';

	/// en-US: 'Keep Existing'
	String get mobileKeepExisting => 'Keep Existing';

	/// en-US: 'Keep Separate'
	String get mobileKeepSeparate => 'Keep Separate';

	/// en-US: 'Keep both apps open.'
	String get mobileKeepBothAppsOpen => 'Keep both apps open.';

	/// en-US: 'Keep this screen open on the same reachable local network.'
	String get mobileKeepThisScreenOpenOnTheSameReachable => 'Keep this screen open on the same reachable local network.';

	/// en-US: 'Kitchen'
	String get mobileKitchen => 'Kitchen';

	/// en-US: 'Living Room'
	String get mobileLivingRoom => 'Living Room';

	/// en-US: 'Living Room \u203a'
	String get mobileLivingRoomE4d208 => 'Living Room  \u203a';

	/// en-US: 'Living Room \u00b7 Drag handles change the visible order.'
	String get mobileLivingRoomDragHandlesChangeTheVisibleOrder => 'Living Room \u00b7 Drag handles change the visible order.';

	/// en-US: 'Living room'
	String get mobileLivingRoom268d40 => 'Living room';

	/// en-US: 'Living room TV'
	String get mobileLivingRoomTv => 'Living room TV';

	/// en-US: 'Living room TV \u00b7 Pairing required'
	String get mobileLivingRoomTvPairingRequired => 'Living room TV \u00b7 Pairing required';

	/// en-US: 'Local access is off'
	String get mobileLocalAccessIsOff => 'Local access is off';

	/// en-US: 'Local authorization and associated cached data will be removed. Access on other devices is not revoked.'
	String get mobileLocalAuthorizationAndAssociatedCachedDataWillBe => 'Local authorization and associated cached data will be removed. Access on other devices is not revoked.';

	/// en-US: 'Local network access lets HomeVerb discover compatible appliances while you use this screen.'
	String get mobileLocalNetworkAccessLetsHomeverbDiscoverCompatibleAppliances => 'Local network access lets HomeVerb discover compatible appliances while you use this screen.';

	/// en-US: 'Local network permission'
	String get mobileLocalNetworkPermission => 'Local network permission';

	/// en-US: 'Missing document export result.'
	String get mobileMissingDocumentExportResult => 'Missing document export result.';

	/// en-US: 'Morgan tablet'
	String get mobileMorganTablet => 'Morgan tablet';

	/// en-US: 'Move 2 Devices'
	String get mobileMove2Devices => 'Move 2 Devices';

	/// en-US: 'Nearby discovery and transfer have stopped.'
	String get mobileNearbyDiscoveryAndTransferHaveStopped => 'Nearby discovery and transfer have stopped.';

	/// en-US: 'No authorization was sent.'
	String get mobileNoAuthorizationWasSent => 'No authorization was sent.';

	/// en-US: 'No devices in Study'
	String get mobileNoDevicesInStudy => 'No devices in Study';

	/// en-US: 'No nearby recipients'
	String get mobileNoNearbyRecipients => 'No nearby recipients';

	/// en-US: 'Normal \u203a'
	String get mobileNormal => 'Normal  \u203a';

	/// en-US: 'Nursery'
	String get mobileNursery => 'Nursery';

	/// en-US: 'Observed target remains 24\u00b0C \u00b7 Read just now'
	String get mobileObservedTargetRemains24CReadJustNow => 'Observed target remains 24\u00b0C \u00b7 Read just now';

	/// en-US: 'On \u00b7 09:40:01'
	String get mobileOn094001 => 'On \u00b7 09:40:01';

	/// en-US: 'On \u00b7 09:40:02'
	String get mobileOn094002 => 'On \u00b7 09:40:02';

	/// en-US: 'Only the first account is eligible. Actual access depends on the platform.'
	String get mobileOnlyTheFirstAccountIsEligibleActualAccess => 'Only the first account is eligible. Actual access depends on the platform.';

	/// en-US: 'Open Living Room'
	String get mobileOpenLivingRoom => 'Open Living Room';

	/// en-US: 'Open a row for evidence and limitations. No real platform is verified by these mockups.'
	String get mobileOpenARowForEvidenceAndLimitationsNo => 'Open a row for evidence and limitations. No real platform is verified by these mockups.';

	/// en-US: 'Pair living room TV'
	String get mobilePairLivingRoomTv => 'Pair living room TV';

	/// en-US: 'Pairing research'
	String get mobilePairingResearch => 'Pairing research';

	/// en-US: 'Pinned \u00b7 \u2261'
	String get mobilePinned446896 => 'Pinned \u00b7 \u2261';

	/// en-US: 'Possibly sent \u00b7 LAN'
	String get mobilePossiblySentLan => 'Possibly sent \u00b7 LAN';

	/// en-US: 'Primary Bedroom'
	String get mobilePrimaryBedroom => 'Primary Bedroom';

	/// en-US: 'Provider-owned browser page; its appearance may differ.'
	String get mobileProviderOwnedBrowserPageItsAppearanceMayDiffer => 'Provider-owned browser page; its appearance may differ.';

	/// en-US: 'Read 09:41:03'
	String get mobileRead094103 => 'Read 09:41:03';

	/// en-US: 'Read 2 min ago'
	String get mobileRead2MinAgo => 'Read 2 min ago';

	/// en-US: 'Read and control all 6 devices'
	String get mobileReadAndControlAll6Devices => 'Read and control all 6 devices';

	/// en-US: 'Ready to receive'
	String get mobileReadyToReceive => 'Ready to receive';

	/// en-US: 'Receive authorization'
	String get mobileReceiveAuthorizationB41058 => 'Receive authorization';

	/// en-US: 'Recipient'
	String get mobileRecipient => 'Recipient';

	/// en-US: 'Recipient opened Receive'
	String get mobileRecipientOpenedReceive => 'Recipient opened Receive';

	/// en-US: 'Remove Imported Authorization'
	String get mobileRemoveImportedAuthorization => 'Remove Imported Authorization';

	/// en-US: 'Replace After Checking'
	String get mobileReplaceAfterChecking => 'Replace After Checking';

	/// en-US: 'Requested 23\u00b0C. Observed target remains 24\u00b0C.'
	String get mobileRequested23CObservedTargetRemains24C => 'Requested 23\u00b0C. Observed target remains 24\u00b0C.';

	/// en-US: 'Resend in 00:28'
	String get mobileResendIn0028 => 'Resend in 00:28';

	/// en-US: 'Review Duplicate'
	String get mobileReviewDuplicate => 'Review Duplicate';

	/// en-US: 'Review authorization'
	String get mobileReviewAuthorization => 'Review authorization';

	/// en-US: 'Revoke Access'
	String get mobileRevokeAccess => 'Revoke Access';

	/// en-US: 'Revoke Access \u00b7 if supported'
	String get mobileRevokeAccessIfSupported => 'Revoke Access \u00b7 if supported';

	/// en-US: 'Revoke platform access?'
	String get mobileRevokePlatformAccess => 'Revoke platform access?';

	/// en-US: 'Room assignment organizes devices; it does not restrict this grant.'
	String get mobileRoomAssignmentOrganizesDevicesItDoesNotRestrict => 'Room assignment organizes devices; it does not restrict this grant.';

	/// en-US: 'Rooms, device names, snapshots and local authorizations will be removed. Other installations and platform access are not revoked.'
	String get mobileRoomsDeviceNamesSnapshotsAndLocalAuthorizationsWill => 'Rooms, device names, snapshots and local authorizations will be removed. Other installations and platform access are not revoked.';

	/// en-US: 'Same reachable network'
	String get mobileSameReachableNetwork => 'Same reachable network';

	/// en-US: 'Scanning \u00b7 01:20 remaining'
	String get mobileScanning0120Remaining => 'Scanning \u00b7 01:20 remaining';

	/// en-US: 'Searching nearby'
	String get mobileSearchingNearby => 'Searching nearby';

	/// en-US: 'Searching on your local network'
	String get mobileSearchingOnYourLocalNetwork => 'Searching on your local network';

	/// en-US: 'Secure import'
	String get mobileSecureImport => 'Secure import';

	/// en-US: 'Select recipient'
	String get mobileSelectRecipient => 'Select recipient';

	/// en-US: 'Selected: Bedroom AC and Air sensor'
	String get mobileSelectedBedroomAcAndAirSensor => 'Selected: Bedroom AC and Air sensor';

	/// en-US: 'Send authorization'
	String get mobileSendAuthorizationF46fe5 => 'Send authorization';

	/// en-US: 'Sending authorization'
	String get mobileSendingAuthorization => 'Sending authorization';

	/// en-US: 'Sent'
	String get mobileSent => 'Sent';

	/// en-US: 'Sharing session ended'
	String get mobileSharingSessionEnded => 'Sharing session ended';

	/// en-US: 'Start Cotton cycle?'
	String get mobileStartCottonCycle => 'Start Cotton cycle?';

	/// en-US: 'Start a New Session'
	String get mobileStartANewSession => 'Start a New Session';

	/// en-US: 'Start again to request a new authorization.'
	String get mobileStartAgainToRequestANewAuthorization => 'Start again to request a new authorization.';

	/// en-US: 'Stop Receiving'
	String get mobileStopReceiving => 'Stop Receiving';

	/// en-US: 'Studio account'
	String get mobileStudioAccount => 'Studio account';

	/// en-US: 'Studio account \u00b7 Sign-in needed'
	String get mobileStudioAccountSignInNeeded => 'Studio account \u00b7 Sign-in needed';

	/// en-US: 'Study'
	String get mobileStudy => 'Study';

	/// en-US: 'Supported action \u203a'
	String get mobileSupportedAction => 'Supported action  \u203a';

	/// en-US: 'Supported: Power \u00b7 Brightness \u00b7 Color temperature'
	String get mobileSupportedPowerBrightnessColorTemperature => 'Supported: Power \u00b7 Brightness \u00b7 Color temperature';

	/// en-US: 'The change may have happened. Last read: 2 min ago.'
	String get mobileTheChangeMayHaveHappenedLastRead2 => 'The change may have happened. Last read: 2 min ago.';

	/// en-US: 'The recipient may have imported the authorization. Ask them to check their result.'
	String get mobileTheRecipientMayHaveImportedTheAuthorizationAsk => 'The recipient may have imported the authorization. Ask them to check their result.';

	/// en-US: 'The search has ended.'
	String get mobileTheSearchHasEnded => 'The search has ended.';

	/// en-US: 'Third-party authorization'
	String get mobileThirdPartyAuthorization => 'Third-party authorization';

	/// en-US: 'This account cannot be shared'
	String get mobileThisAccountCannotBeShared => 'This account cannot be shared';

	/// en-US: 'This device: Home phone'
	String get mobileThisDeviceHomePhone => 'This device: Home phone';

	/// en-US: 'This grant cannot be limited to selected devices.'
	String get mobileThisGrantCannotBeLimitedToSelectedDevices => 'This grant cannot be limited to selected devices.';

	/// en-US: 'This phone now connects independently.'
	String get mobileThisPhoneNowConnectsIndependently => 'This phone now connects independently.';

	/// en-US: 'Today 09:40'
	String get mobileToday0940 => 'Today 09:40';

	/// en-US: 'Today 09:42'
	String get mobileToday0942 => 'Today 09:42';

	/// en-US: 'Transfer'
	String get mobileTransfer => 'Transfer';

	/// en-US: 'Transfer result'
	String get mobileTransferResult => 'Transfer result';

	/// en-US: 'Transferred'
	String get mobileTransferred => 'Transferred';

	/// en-US: 'UI preview \u00b7 Tap to browse states \u00b7 No native operations'
	String get mobileUiPreviewTapToBrowseStatesNoNative => 'UI preview \u00b7 Tap to browse states \u00b7 No native operations';

	/// en-US: 'Unassigned \u203a'
	String get mobileUnassigned => 'Unassigned  \u203a';

	/// en-US: 'Unknown \u00b7 09:42'
	String get mobileUnknown0942 => 'Unknown \u00b7 09:42';

	/// en-US: 'Use your local network \u203a'
	String get mobileUseYourLocalNetwork => 'Use your local network  \u203a';

	/// en-US: 'Verified \u00b7 09:40'
	String get mobileVerified0940 => 'Verified \u00b7 09:40';

	/// en-US: 'Verified \u00b7 On'
	String get mobileVerifiedOn => 'Verified \u00b7 On';

	/// en-US: 'Verify Example Home'
	String get mobileVerifyExampleHome => 'Verify Example Home';

	/// en-US: 'Verify recipient'
	String get mobileVerifyRecipient => 'Verify recipient';

	/// en-US: 'Waiting for approval'
	String get mobileWaitingForApproval => 'Waiting for approval';

	/// en-US: 'Washing machine'
	String get mobileWashingMachine => 'Washing machine';

	/// en-US: 'Window Light \u203a'
	String get mobileWindowLight329ec1 => 'Window Light  \u203a';

	/// en-US: 'Yesterday 20:14'
	String get mobileYesterday2014 => 'Yesterday 20:14';

	/// en-US: 'You can change their local rooms.'
	String get mobileYouCanChangeTheirLocalRooms => 'You can change their local rooms.';

	/// en-US: 'Direct cloud connection. Supported devices use read-only access in this build.'
	String get mobileDirectCloudConnectionSupportedDevicesUseReadOnly472707 => 'Direct cloud connection. Supported devices use read-only access in this build.';

	/// en-US: '{phase} \u00b7 {transport}{suffix}'
	String mobilePhaseTransportSuffix({required Object phase, required Object transport, required Object suffix}) => '${phase} \u00b7 ${transport}${suffix}';

	/// en-US: ' (retained)'
	String get mobileRetained => ' (retained)';

	/// en-US: 'LAN'
	String get mobileLan => 'LAN';

	/// en-US: '{value}{stale}'
	String mobileValueStaleEdf502({required Object value, required Object stale}) => '${value}${stale}';

	/// en-US: 'HOME'
	String get mobileSectionHome => 'HOME';

	/// en-US: 'PREFERENCES'
	String get mobileSectionPreferences => 'PREFERENCES';

	/// en-US: 'Checking Android credential protection\u2026'
	String get mobileCredentialProtectionChecking => 'Checking Android credential protection\u2026';

	/// en-US: 'Set a screen-lock PIN, pattern or password in Android Settings, then unlock this device and return. HomeVerb cannot save authorization without a secure screen lock.'
	String get mobileCredentialScreenLockRequired => 'Set a screen-lock PIN, pattern or password in Android Settings, then unlock this device and return. HomeVerb cannot save authorization without a secure screen lock.';

	/// en-US: 'Unlock this device, then return to sign in.'
	String get mobileCredentialDeviceLocked => 'Unlock this device, then return to sign in.';

	/// en-US: 'Android credential support is unavailable. Restart the app or install a matching native build before trying again.'
	String get mobileCredentialBackendUnavailable => 'Android credential support is unavailable. Restart the app or install a matching native build before trying again.';

	/// en-US: 'Open Android Security Settings'
	String get mobileCredentialOpenSettings => 'Open Android Security Settings';

	/// en-US: 'Could not open Android security settings. Open them from the system Settings app.'
	String get mobileCredentialSettingsUnavailable => 'Could not open Android security settings. Open them from the system Settings app.';

	/// en-US: 'Check Again'
	String get mobileCredentialCheckAgain => 'Check Again';

	/// en-US: 'Direct cloud connection. Open a supported device to use its available controls.'
	String get mobileDefaultControlsMessage1 => 'Direct cloud connection. Open a supported device to use its available controls.';

	/// en-US: 'Use your existing Meiju account. Sign-in retrieves bound devices automatically. Open a supported device to control it.'
	String get mobileDefaultControlsMessage2 => 'Use your existing Meiju account. Sign-in retrieves bound devices automatically. Open a supported device to control it.';

	/// en-US: 'Sign in to the original account. Identity must match; local device names, rooms and favorites are preserved. Open a supported device to control it.'
	String get mobileDefaultControlsMessage3 => 'Sign in to the original account. Identity must match; local device names, rooms and favorites are preserved. Open a supported device to control it.';

	/// en-US: '0.2.0 \u00b7 Experimental Meiju \u00b7 Device controls'
	String get mobileDefaultControlsMessage4 => '0.2.0 \u00b7 Experimental Meiju \u00b7 Device controls';

	/// en-US: 'Supported controls are available when you open a device. Compatibility depends on its exact model and region.'
	String get mobileDefaultControlsMessage5 => 'Supported controls are available when you open a device. Compatibility depends on its exact model and region.';

	/// en-US: 'Experimental device support'
	String get mobileDefaultControlsMessage6 => 'Experimental device support';

	/// en-US: 'Air direction'
	String get mobileAirDirection => 'Air direction';

	/// en-US: 'Not supported yet'
	String get mobileNotSupportedYet => 'Not supported yet';

	/// en-US: 'Selecting a mode also turns this device on.'
	String get mobileSelectingModeTurnsOn => 'Selecting a mode also turns this device on.';

	/// en-US: 'Updating device\u2026'
	String get mobileUpdatingDevice => 'Updating device\u2026';

	/// en-US: 'Unrecognized ({value})'
	String mobileUnrecognizedControlValue({required Object value}) => 'Unrecognized (${value})';

	/// en-US: 'Function support'
	String get mobileClimateFunctionSupport => 'Function support';

	/// en-US: 'This list describes HomeVerb integration, not the appliance hardware. Unconnected functions cannot send commands.'
	String get mobileClimateSupportExplanation => 'This list describes HomeVerb integration, not the appliance hardware. Unconnected functions cannot send commands.';

	/// en-US: 'Not connected in HomeVerb'
	String get mobileClimateNotConnected => 'Not connected in HomeVerb';

	/// en-US: 'Temperature control is not connected yet.'
	String get mobileClimateTemperatureNotConnected => 'Temperature control is not connected yet.';

	/// en-US: 'Indoor {temperature}'
	String mobileClimateIndoorReading({required Object temperature}) => 'Indoor ${temperature}';

	/// en-US: 'More functions'
	String get mobileClimateMoreFunctions => 'More functions';

	/// en-US: 'Vertical swing'
	String get mobileClimateVerticalSwing => 'Vertical swing';

	/// en-US: 'Horizontal swing'
	String get mobileClimateHorizontalSwing => 'Horizontal swing';

	/// en-US: 'Avoid direct airflow'
	String get mobileClimateAvoidDirectAirflow => 'Avoid direct airflow';

	/// en-US: 'Sleep'
	String get mobileClimateSleep => 'Sleep';

	/// en-US: 'Energy saving'
	String get mobileClimateEnergySaving => 'Energy saving';

	/// en-US: 'Boost'
	String get mobileClimateBoost => 'Boost';

	/// en-US: 'Display'
	String get mobileClimateDisplay => 'Display';

	/// en-US: 'Auxiliary heat'
	String get mobileClimateAuxiliaryHeat => 'Auxiliary heat';

	/// en-US: 'Timer'
	String get mobileClimateTimer => 'Timer';

	/// en-US: 'Self-clean'
	String get mobileClimateSelfClean => 'Self-clean';

	/// en-US: 'Available controls'
	String get mobileClimateAvailableControls => 'Available controls';

	/// en-US: 'Other settings'
	String get mobileClimateOtherSettings => 'Other settings';

	/// en-US: 'Horizontal direction'
	String get mobileClimateHorizontalDirection => 'Horizontal direction';

	/// en-US: 'Vertical direction'
	String get mobileClimateVerticalDirection => 'Vertical direction';

	/// en-US: 'Display light'
	String get mobileClimateDisplayLight => 'Display light';

	/// en-US: 'Button sound'
	String get mobileClimateButtonSound => 'Button sound';

	/// en-US: 'Accounts'
	String get mobilePlatformAccounts => 'Accounts';

	/// en-US: 'Optional. Uses your sign-in account when blank.'
	String get mobileOptionalAccountRemark => 'Optional. Uses your sign-in account when blank.';

	/// en-US: 'Use at most 80 characters without control characters.'
	String get mobileOptionalAccountRemarkValidation => 'Use at most 80 characters without control characters.';

	/// en-US: 'Energy saving'
	String get vendorMideaAcEco => 'Energy saving';

	/// en-US: 'Comfort energy saving'
	String get vendorMideaAcComfortEnergySaving => 'Comfort energy saving';

	/// en-US: 'Comfort energy saving'
	String get mobileAcFeatureComfortEnergySaving => 'Comfort energy saving';

	/// en-US: 'Breezeless'
	String get vendorMideaAcBreezeless => 'Breezeless';

	/// en-US: 'Breezeless'
	String get mobileAcFeatureBreezeless => 'Breezeless';

	/// en-US: 'Horizontal swing'
	String get vendorMideaAcHorizontalSwing => 'Horizontal swing';

	/// en-US: 'Vertical swing'
	String get vendorMideaAcVerticalSwing => 'Vertical swing';

	/// en-US: 'Lower horizontal swing'
	String get vendorMideaAcLowerHorizontalSwing => 'Lower horizontal swing';

	/// en-US: 'Lower horizontal swing'
	String get mobileAcFeatureLowerHorizontalSwing => 'Lower horizontal swing';

	/// en-US: 'Horizontal direction'
	String get vendorMideaAcHorizontalDirection => 'Horizontal direction';

	/// en-US: 'Vertical direction'
	String get vendorMideaAcVerticalDirection => 'Vertical direction';

	/// en-US: 'Display light'
	String get vendorMideaAcDisplay => 'Display light';

	/// en-US: 'Button sound'
	String get vendorMideaAcButtonSound => 'Button sound';

	/// en-US: 'Sleep'
	String get vendorMideaAcSleep => 'Sleep';

	/// en-US: 'Boost'
	String get vendorMideaAcBoost => 'Boost';

	/// en-US: 'Avoid direct airflow'
	String get vendorMideaAcAvoidDirectAirflow => 'Avoid direct airflow';

	/// en-US: 'Prevent overcooling'
	String get vendorMideaAcPreventOvercooling => 'Prevent overcooling';

	/// en-US: 'Prevent overcooling'
	String get mobileAcFeaturePreventOvercooling => 'Prevent overcooling';

	/// en-US: 'Dry after shutdown'
	String get vendorMideaAcDryAfterShutdown => 'Dry after shutdown';

	/// en-US: 'Dry after shutdown'
	String get mobileAcFeatureDryAfterShutdown => 'Dry after shutdown';

	/// en-US: 'Auxiliary heat'
	String get vendorMideaAcAuxiliaryHeat => 'Auxiliary heat';

	/// en-US: 'Follow airflow'
	String get vendorMideaAcFollowAirflow => 'Follow airflow';

	/// en-US: 'Follow airflow'
	String get mobileAcFeatureFollowAirflow => 'Follow airflow';

	/// en-US: 'Odor removal'
	String get vendorMideaAcFreshAirRemoveOdor => 'Odor removal';

	/// en-US: 'Odor removal'
	String get mobileAcFeatureFreshAirRemoveOdor => 'Odor removal';

	/// en-US: 'Odor removal'
	String get vendorMideaAcFengguanRemoveOdor => 'Odor removal';

	/// en-US: 'Odor removal'
	String get mobileAcFeatureFengguanRemoveOdor => 'Odor removal';

	/// en-US: 'Electrostatic dust removal'
	String get vendorMideaAcElecDustRemove => 'Electrostatic dust removal';

	/// en-US: 'Electrostatic dust removal'
	String get mobileAcFeatureElecDustRemove => 'Electrostatic dust removal';

	/// en-US: 'Automatic fresh air'
	String get vendorMideaAcAutoComfortFreshAir => 'Automatic fresh air';

	/// en-US: 'Automatic fresh air'
	String get mobileAcFeatureAutoComfortFreshAir => 'Automatic fresh air';

	/// en-US: 'Fresh air CO2 control'
	String get vendorMideaAcAutoFreshOffCo2 => 'Fresh air CO2 control';

	/// en-US: 'Fresh air CO2 control'
	String get mobileAcFeatureAutoFreshOffCo2 => 'Fresh air CO2 control';

	/// en-US: 'Fresh air'
	String get vendorMideaAcComfortFreshAir => 'Fresh air';

	/// en-US: 'Fresh air'
	String get mobileAcFeatureComfortFreshAir => 'Fresh air';

	/// en-US: 'Fresh air fan speed'
	String get vendorMideaAcFreshAirFanSpeed => 'Fresh air fan speed';

	/// en-US: 'Fresh air fan speed'
	String get mobileAcFeatureFreshAirFanSpeed => 'Fresh air fan speed';

	/// en-US: 'Manual humidity'
	String get vendorMideaAcManulHumi => 'Manual humidity';

	/// en-US: 'Manual humidity'
	String get mobileAcFeatureManulHumi => 'Manual humidity';

	/// en-US: 'Target humidity'
	String get vendorMideaAcManulHumiValue => 'Target humidity';

	/// en-US: 'Target humidity'
	String get mobileAcFeatureManulHumiValue => 'Target humidity';

	/// en-US: 'Purifier PM threshold'
	String get vendorMideaAcAutoPurifierOnPm => 'Purifier PM threshold';

	/// en-US: 'Purifier PM threshold'
	String get mobileAcFeatureAutoPurifierOnPm => 'Purifier PM threshold';

	/// en-US: 'Off'
	String get vendorMideaAcBreezeless0 => 'Off';

	/// en-US: 'Upper and lower breezeless'
	String get vendorMideaAcBreezeless1 => 'Upper and lower breezeless';

	/// en-US: 'Upper breezeless'
	String get vendorMideaAcBreezeless2 => 'Upper breezeless';

	/// en-US: 'Lower breezeless'
	String get vendorMideaAcBreezeless3 => 'Lower breezeless';

	/// en-US: 'Fixed direction off'
	String get vendorMideaAcHorizontalDirection0 => 'Fixed direction off';

	/// en-US: 'Far left'
	String get vendorMideaAcHorizontalDirection1 => 'Far left';

	/// en-US: 'Left'
	String get vendorMideaAcHorizontalDirection25 => 'Left';

	/// en-US: 'Middle'
	String get vendorMideaAcHorizontalDirection50 => 'Middle';

	/// en-US: 'Right'
	String get vendorMideaAcHorizontalDirection75 => 'Right';

	/// en-US: 'Far right'
	String get vendorMideaAcHorizontalDirection100 => 'Far right';

	/// en-US: 'Fixed direction off'
	String get vendorMideaAcVerticalDirection0 => 'Fixed direction off';

	/// en-US: 'Top'
	String get vendorMideaAcVerticalDirection1 => 'Top';

	/// en-US: 'Upper'
	String get vendorMideaAcVerticalDirection25 => 'Upper';

	/// en-US: 'Middle'
	String get vendorMideaAcVerticalDirection50 => 'Middle';

	/// en-US: 'Lower'
	String get vendorMideaAcVerticalDirection75 => 'Lower';

	/// en-US: 'Bottom'
	String get vendorMideaAcVerticalDirection100 => 'Bottom';

	/// en-US: 'Low'
	String get vendorMideaAcFreshAirFanSpeed40 => 'Low';

	/// en-US: 'Medium'
	String get vendorMideaAcFreshAirFanSpeed60 => 'Medium';

	/// en-US: 'High'
	String get vendorMideaAcFreshAirFanSpeed80 => 'High';

	/// en-US: 'Full'
	String get vendorMideaAcFreshAirFanSpeed100 => 'Full';

	/// en-US: 'Add a platform to bring your appliances here.'
	String get mobilePlatformEmpty => 'Add a platform to bring your appliances here.';

	/// en-US: 'Choose the app you already use for your appliances.'
	String get mobilePlatformChoose => 'Choose the app you already use for your appliances.';

	/// en-US: 'Available platforms'
	String get mobilePlatformAvailable => 'Available platforms';

	/// en-US: 'Supported devices vary by model.'
	String get mobilePlatformModelSupport => 'Supported devices vary by model.';

	/// en-US: 'Device list updated'
	String get mobilePlatformUpdated => 'Device list updated';

	/// en-US: 'Some devices could not be updated'
	String get mobilePlatformPartial => 'Some devices could not be updated';

	/// en-US: 'Could not update devices'
	String get mobilePlatformFailed => 'Could not update devices';

	/// en-US: '{count} devices found.'
	String mobilePlatformFound({required Object count}) => '${count} devices found.';

	/// en-US: 'Your saved devices are still here. Try again or sign in again.'
	String get mobilePlatformRetry => 'Your saved devices are still here. Try again or sign in again.';

	/// en-US: 'Check that your appliances are added to this account in Meiju.'
	String get mobilePlatformCheckMeiju => 'Check that your appliances are added to this account in Meiju.';

	/// en-US: 'Finish removing this account from this device.'
	String get mobilePlatformFinishRemoval => 'Finish removing this account from this device.';

	/// en-US: 'Restore access to update your devices.'
	String get mobilePlatformRestore => 'Restore access to update your devices.';

	/// en-US: 'No devices yet. Update the list after adding appliances in Meiju.'
	String get mobilePlatformNoDevices => 'No devices yet. Update the list after adding appliances in Meiju.';

	/// en-US: 'Manage account'
	String get mobilePlatformManage => 'Manage account';

	/// en-US: 'Troubleshooting'
	String get mobilePlatformTroubleshooting => 'Troubleshooting';

	/// en-US: 'This only removes the account and its devices from HomeVerb on this device. Your Meiju account is unchanged. Reconnecting will not restore device names or room assignments.'
	String get mobilePlatformRemovalScope => 'This only removes the account and its devices from HomeVerb on this device. Your Meiju account is unchanged. Reconnecting will not restore device names or room assignments.';

	/// en-US: 'Sign in to the same Meiju account. Your device names and rooms will stay.'
	String get mobilePlatformSignInAgain => 'Sign in to the same Meiju account. Your device names and rooms will stay.';

	/// en-US: 'Sign in with your Meiju account to add your appliances.'
	String get mobilePlatformSignIn => 'Sign in with your Meiju account to add your appliances.';

	/// en-US: 'Refresh Devices'
	String get mobileRefreshAccountDevices => 'Refresh Devices';

	/// en-US: 'Reported program'
	String get vendorMideaDishwasherMode => 'Reported program';

	/// en-US: 'Cycle status'
	String get vendorMideaDishwasherWorkStatus => 'Cycle status';

	/// en-US: 'Door status'
	String get vendorMideaDishwasherDoorswitch => 'Door status';

	/// en-US: 'Water supply warning'
	String get vendorMideaDishwasherWaterLack => 'Water supply warning';

	/// en-US: 'Salt warning'
	String get vendorMideaDishwasherSoftwaterLack => 'Salt warning';

	/// en-US: 'Rinse aid warning'
	String get vendorMideaDishwasherBrightLack => 'Rinse aid warning';

	/// en-US: 'Temperature'
	String get vendorMideaDishwasherTemperature => 'Temperature';

	/// en-US: 'Remaining time'
	String get vendorMideaDishwasherRemainingMinutes => 'Remaining time';
}

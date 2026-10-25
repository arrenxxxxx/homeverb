#ifndef HOMEVERB_H
#define HOMEVERB_H
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* All strings are UTF-8. Inputs remain caller-owned; free each output exactly once. */
/* Call open/call/connect/approval/scan/link/close on worker threads. Cancel and foreground updates may run concurrently. */
char *homeverb_open(const char *config_json);
char *homeverb_call(uint64_t handle, const char *request_json);
/* Secret buffers are disjoint caller-owned writable memory; no concurrent access. */
/* Native code clears valid 1..256-byte buffers before admission. Always clear/free them. */
/* API 1.24+: optional provider selects a registered flow; credentials remain separate. */
char *homeverb_connect_account(uint64_t handle, const char *metadata_json,
                              uint8_t *login, size_t login_length,
                              uint8_t *password, size_t password_length);
/* API 1.25+: trusted legacy-account recovery; preparation is not confirmation. */
/* Use the same protected buffer rules as connect. Show account/count and obtain */
/* actual same-account consent. Confirm closes this host; reopen read-only. */
/* API 1.28: protected Mijia interactive sign-in; never ordinary JSON tools.
 * Input buffers follow homeverb_connect_account ownership/clearing rules.
 * Answer buffers must be writable, nonoverlapping and 1..256 bytes; valid
 * answers are 1..32 ASCII alphanumeric bytes. Results use homeverb_string_free.
 * Captcha image data is for the trusted UI only; do not log or persist it. */
char *homeverb_begin_account_login(uint64_t handle, const char *metadata_json,
    uint8_t *login, size_t login_length, uint8_t *password, size_t password_length);
char *homeverb_continue_account_login(uint64_t handle, const char *metadata_json,
    uint8_t *answer, size_t answer_length);
bool homeverb_discard_account_login(uint64_t handle, uint64_t challenge_id);

char *homeverb_prepare_identity_recovery(uint64_t handle, const char *metadata_json,
                                       uint8_t *login, size_t login_length,
                                       uint8_t *password, size_t password_length);
char *homeverb_confirm_identity_recovery(uint64_t handle, const char *request_json);
bool homeverb_discard_identity_recovery(uint64_t handle, uint64_t recovery_id);
/* API 1.13+: trusted user interaction only; never expose these as model tools. */
/* Display the immutable preview, then execute only after explicit user consent. */
/* Pending IDs are local to this handle, expire, and are cleared on background/close. */
char *homeverb_prepare_approval(uint64_t handle, const char *request_json);
char *homeverb_execute_approved(uint64_t handle, const char *request_json);
bool homeverb_discard_approval(uint64_t handle, uint64_t approval_id);
/* API 1.18+: trusted UI scan consent, never ordinary JSON/model tools. */
/* Obtain assigned interface/permission from the platform. IPv4 is numeric octets: */
/* 192.168.42.2 = 0xC0A82A02 regardless of memory byte order. Display returned scope. */
char *homeverb_prepare_lan_scan(uint64_t handle, const char *request_json,
                               uint32_t local_address, uint8_t prefix_length);
/* Recheck interface/permission and obtain user consent before consuming the preview. */
char *homeverb_confirm_lan_scan(uint64_t handle, const char *request_json);
bool homeverb_discard_lan_scan(uint64_t handle, uint64_t scan_id);
/* API 1.20+: trusted connection-link preview, actual user consent and dismissal. */
/* Adapters must independently admit protected evidence; Mock/Meiju currently refuse. */
/* Show the exact descriptor, anchor, proposed connection and potential transport grants. */
char *homeverb_prepare_connection_link(uint64_t handle, const char *request_json);
char *homeverb_confirm_connection_link(uint64_t handle, const char *request_json);
bool homeverb_discard_connection_link(uint64_t handle, uint64_t link_id);
bool homeverb_cancel(uint64_t handle, const char *operation_id);
void homeverb_set_foreground(uint64_t handle, bool foreground);
void homeverb_close(uint64_t handle);
void homeverb_string_free(char *value);
#endif

#ifndef HOMEVERB_H
#define HOMEVERB_H
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* All strings are UTF-8. Inputs remain caller-owned; free each output exactly once. */
/* Call open/call/close on worker threads. Cancel and foreground updates may run concurrently. */
char *homeverb_open(const char *config_json);
char *homeverb_call(uint64_t handle, const char *request_json);
/* Secret buffers are disjoint caller-owned writable memory; no concurrent access. */
/* Native code clears valid 1..256-byte buffers before admission. Always clear/free them. */
char *homeverb_connect_account(uint64_t handle, const char *metadata_json,
                              uint8_t *login, size_t login_length,
                              uint8_t *password, size_t password_length);
bool homeverb_cancel(uint64_t handle, const char *operation_id);
void homeverb_set_foreground(uint64_t handle, bool foreground);
void homeverb_close(uint64_t handle);
void homeverb_string_free(char *value);
#endif

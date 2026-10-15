#ifndef HOMEVERB_H
#define HOMEVERB_H
#include <stdbool.h>
#include <stdint.h>

/* All strings are UTF-8. Inputs remain caller-owned; free each output exactly once. */
/* Call open/call/close on worker threads. Cancel and foreground updates may run concurrently. */
char *homeverb_open(const char *config_json);
char *homeverb_call(uint64_t handle, const char *request_json);
bool homeverb_cancel(uint64_t handle, const char *operation_id);
void homeverb_set_foreground(uint64_t handle, bool foreground);
void homeverb_close(uint64_t handle);
void homeverb_string_free(char *value);
#endif

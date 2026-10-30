/* Fictional API peer for negotiation before event-history startup. */
#ifndef FIXTURE_API_VERSION
#define FIXTURE_API_VERSION "1.44.0"
#endif
#ifndef FIXTURE_METHODS
#define FIXTURE_METHODS "\"inspect_event_history_options\":{},\"read_event_history\":{}"
#endif
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
static unsigned opened, enabled, closed;
unsigned fixture_opened(void) { return opened; }
unsigned fixture_enabled(void) { return enabled; }
unsigned fixture_closed(void) { return closed; }
char *homeverb_open(const char *config) {
    opened++;
    if (strstr(config, "mijia_event_history")) enabled++;
    return strdup("{\"schema_version\":\"1.0\",\"data\":{\"handle\":1,\"adapter\":\"xiaomi.mijia\"},\"warnings\":[],\"error\":null}");
}
char *homeverb_call(uint64_t handle, const char *request) {
    (void)handle; (void)request;
    return strdup("{\"schema_version\":\"1.0\",\"data\":{\"api_version\":\"" FIXTURE_API_VERSION "\",\"methods\":{" FIXTURE_METHODS "}},\"warnings\":[],\"error\":null}");
}
void homeverb_close(uint64_t handle) { (void)handle; closed++; }
void homeverb_string_free(char *value) { free(value); }
bool homeverb_cancel(uint64_t handle, const char *operation) { (void)handle; (void)operation; return false; }
void homeverb_set_foreground(uint64_t handle, bool value) { (void)handle; (void)value; }

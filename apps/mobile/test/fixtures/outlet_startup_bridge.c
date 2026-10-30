/* Fictional older API peer: additive grants must never reach homeverb_open. */
#ifndef FIXTURE_API_VERSION
#define FIXTURE_API_VERSION "1.33.0"
#endif
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static unsigned opened, granted, closed;
unsigned fixture_opened(void) { return opened; }
unsigned fixture_granted(void) { return granted; }
unsigned fixture_closed(void) { return closed; }
char *homeverb_open(const char *config) {
    opened++;
    if ((strstr(config, "mijia_outlet_grants") || strstr(config, "mijia_setting_grants") || strstr(config, "mijia_action_grants"))) granted++;
    return strdup("{\"schema_version\":\"1.0\",\"data\":{\"handle\":1,\"adapter\":\"xiaomi.mijia\"},\"warnings\":[],\"error\":null}");
}
char *homeverb_call(uint64_t handle, const char *request) {
    (void)handle; (void)request;
    return strdup("{\"schema_version\":\"1.0\",\"data\":{\"api_version\":\"" FIXTURE_API_VERSION "\",\"methods\":{}},\"warnings\":[],\"error\":null}");
}
void homeverb_close(uint64_t handle) { (void)handle; closed++; }
void homeverb_string_free(char *value) { free(value); }
bool homeverb_cancel(uint64_t handle, const char *operation) { (void)handle; (void)operation; return false; }
void homeverb_set_foreground(uint64_t handle, bool value) { (void)handle; (void)value; }

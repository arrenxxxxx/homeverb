/* Fictional ABI fixture: no networking, accounts, platform store or device access. */
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <unistd.h>
static atomic_int mode, entered, release_call, discarded;
void fixture_mode(int value) { mode = value; entered = 0; release_call = 0; discarded = 0; }
int fixture_entered(void) { return entered; }
int fixture_discarded(void) { return discarded; }
void fixture_release(void) { release_call = 1; }
static char *response(const char *data) {
  const char *prefix = "{\"error\":null,\"data\":";
  size_t size = strlen(prefix) + strlen(data) + 2;
  char *out = malloc(size);
  strcpy(out, prefix); strcat(out, data); strcat(out, "}"); return out;
}
static char *failure(void) { return strdup("{\"error\":{\"code\":\"FIXTURE_MISMATCH\",\"message\":\"Invalid fixture ABI input.\"}}"); }
char *homeverb_open(const char *metadata) {
  (void)metadata; return response("{\"handle\":1,\"adapter\":\"xiaomi.mijia\"}");
}
char *homeverb_call(uint64_t handle, const char *metadata) {
  (void)handle; (void)metadata;
  return response("{\"api_version\":\"1.32.0\",\"methods\":{\"list_accounts\":{},\"refresh_account\":{}}}");
}
void homeverb_string_free(char *value) { free(value); }
void homeverb_close(uint64_t handle) { (void)handle; }
void homeverb_set_foreground(uint64_t handle, bool foreground) { (void)handle; (void)foreground; }
bool homeverb_cancel(uint64_t handle, const char *operation) { (void)handle; (void)operation; return true; }
bool homeverb_discard_account_verification(uint64_t handle, uint64_t challenge) {
  (void)handle; (void)challenge; discarded++; return true;
}
static char *prompt(uint8_t *out, size_t capacity, bool sent) {
  if (capacity != 131072) return failure();
  /* Deliberately invalid UTF-8: the protected provider page must not enter Dart strings. */
  memset(out, 255, capacity);
  if (mode == 2) return response("{\"kind\":\"verification\",\"challenge_id\":77,\"prompt_length\":8193,\"expires_in_ms\":1000,\"methods\":[\"phone\"]}");
  if (mode == 3) return response("{\"kind\":\"verification\",\"challenge_id\":77,\"prompt_length\":8,\"expires_in_ms\":1000,\"methods\":[\"phone\",\"phone\"]}");
  if (mode == 4) return response("{\"kind\":\"verification\",\"challenge_id\":77,\"prompt_length\":8,\"expires_in_ms\":1000,\"methods\":[\"phone\"],\"url\":\"fictional\"}");
  if (sent) return response("{\"kind\":\"verification\",\"challenge_id\":3,\"prompt_length\":8,\"expires_in_ms\":170000,\"methods\":[\"phone\",\"email\"],\"code_requests\":1,\"cooldown_ms\":60000}");
  return response("{\"kind\":\"verification\",\"challenge_id\":2,\"prompt_length\":8,\"expires_in_ms\":175000,\"methods\":[\"phone\",\"email\"]}");
}
char *homeverb_begin_account_verification(uint64_t handle, const char *metadata,
    uint8_t *login, size_t login_size, uint8_t *password, size_t password_size,
    uint8_t *out, size_t capacity) {
  if (handle != 1 || !strstr(metadata, "xiaomi.mijia") || login_size != 7 || password_size != 8 ||
      memcmp(login, "fiction", 7) || memcmp(password, "Fiction!", 8) || capacity != 131072) return failure();
  if (mode >= 2 && mode <= 4) return prompt(out, capacity, false);
  if (mode == 5) { entered = 1; while (!release_call) usleep(1000); }
  memcpy(out, "fictional-image", 15);
  return response("{\"kind\":\"captcha\",\"challenge_id\":1,\"prompt_length\":15,\"expires_in_ms\":179000,\"media_type\":\"image/png\"}");
}
char *homeverb_continue_account_verification(uint64_t handle, const char *metadata,
    uint8_t *answer, size_t size, uint8_t *out, size_t capacity) {
  if (handle != 1 || capacity != 131072) return failure();
  if (strstr(metadata, "\"answer_kind\":\"captcha\"")) {
    if (size != 4 || memcmp(answer, "Ab12", 4)) return failure();
    return prompt(out, capacity, false);
  }
  if (!strstr(metadata, "\"answer_kind\":\"email\"") || size != 6 || memcmp(answer, "123456", 6)) return failure();
  return response("{\"kind\":\"connected\",\"report\":{\"fictional\":true}}");
}
char *homeverb_request_account_verification_code(uint64_t handle, const char *metadata,
    uint8_t *out, size_t capacity) {
  if (handle != 1 || !strstr(metadata, "\"method\":\"email\"") || !strstr(metadata, "\"challenge_id\":2")) return failure();
  return prompt(out, capacity, true);
}

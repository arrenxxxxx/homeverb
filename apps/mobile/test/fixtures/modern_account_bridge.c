/* Fictional ABI only; no networking, system credentials or appliances. */
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <unistd.h>
static atomic_int mode, entered, release_call, discarded, begins, continues, sessions;
void fixture_mode(int value) { mode=value; entered=0; release_call=0; discarded=0; begins=0; continues=0; sessions=0; }
void fixture_release(void) { release_call=1; }
int fixture_entered(void) { return entered; }
int fixture_discarded(void) { return discarded; }
int fixture_begins(void) { return begins; }
int fixture_continues(void) { return continues; }
int fixture_sessions(void) { return sessions; }
static char *response(const char *data) {
  size_t size=strlen(data)+48; char *result=malloc(size);
  snprintf(result,size,"{\"error\":null,\"data\":%s}",data); return result;
}
static char *failure(void) { return strdup("{\"error\":{\"code\":\"FIXTURE_MISMATCH\",\"message\":\"Invalid fixture input.\"}}"); }
char *homeverb_open(const char *metadata) { (void)metadata; return response("{\"handle\":1,\"adapter\":\"xiaomi.mijia\"}"); }
char *homeverb_call(uint64_t handle,const char *metadata) { (void)handle; (void)metadata;
#ifdef OMIT_MODERN
  return response("{\"api_version\":\"1.35.0\",\"methods\":{\"list_accounts\":{},\"refresh_account\":{}}}");
#elif defined(PUBLISH_DEVICES)
  return response("{\"api_version\":\"1.37.0\",\"methods\":{\"list_accounts\":{},\"refresh_account\":{}}}");
#else
  return response("{\"api_version\":\"1.36.0\",\"methods\":{\"list_accounts\":{},\"refresh_account\":{}}}");
#endif
}
void homeverb_string_free(char *p) { free(p); }
void homeverb_close(uint64_t handle) { (void)handle; }
void homeverb_set_foreground(uint64_t handle,bool foreground) { (void)handle; (void)foreground; }
bool homeverb_cancel(uint64_t handle,const char *operation) { (void)handle; (void)operation; return true; }
#ifndef OMIT_MODERN
bool homeverb_discard_modern_account_login(uint64_t handle,uint64_t id) { (void)handle; (void)id; discarded++; return true; }
static void stall(int phase) { if (mode==phase) { entered=1; while (!release_call) usleep(1000); } }
char *homeverb_begin_modern_account_login(uint64_t handle,const char *metadata,uint8_t *scan,size_t capacity) {
  if (handle!=1 || capacity!=8192 || !strstr(metadata,"\"timeout_ms\":180000") || !strstr(metadata,"xiaomi.mijia")) return failure();
  begins++; stall(5); memset(scan,0,capacity); memcpy(scan,"https://fictional.invalid/PRIVATE",33);
  if (mode==1) return response("{\"kind\":\"qr\",\"challenge_id\":1,\"scan_length\":8193,\"expires_in_ms\":180000}");
  if (mode==2) return response("{\"kind\":\"qr\",\"challenge_id\":1,\"scan_length\":33,\"expires_in_ms\":0}");
  if (mode==3) return response("{\"kind\":\"qr\",\"challenge_id\":1,\"scan_length\":33,\"expires_in_ms\":180000,\"url\":\"PRIVATE\"}");
  if (mode==4) scan[0]=255;
  if (mode==14) return response("{\"kind\":\"qr\",\"challenge_id\":1,\"scan_length\":33,\"expires_in_ms\":1}");
  return response("{\"kind\":\"qr\",\"challenge_id\":1,\"scan_length\":33,\"expires_in_ms\":179000}");
}
static char *result(bool renewed,bool connected) {
  char account[768], data[1600];
  snprintf(account,sizeof(account),"{\"id\":\"00000000-0000-4000-8000-00000000000%s\",\"provider\":\"%s\",\"region\":\"CN\",\"label\":\"Home\",\"credential_state\":\"stored\",\"authorization\":\"not_checked\",\"last_complete_inventory_at\":null}",mode==11?"2":"1",mode==8?"midea.meiju":"xiaomi.mijia");

#ifdef PUBLISH_DEVICES
  if (!renewed) {
    char *ids=calloc(200000,1), *projected=calloc(220000,1);
    int count=(mode==12 || mode==22)?0:mode==15?2:mode==20?4097:1;
    size_t length=0;
    for (int i=0;i<count;i++) {
      if (mode==10) length+=(size_t)snprintf(ids+length,200000-length,"%s\"invalid-id\"",i?",":"");
      else length+=(size_t)snprintf(ids+length,200000-length,"%s\"40000000-0000-4000-8000-%012x\"",i?",":"",mode==15?1:i+1);
    }
    snprintf(projected,220000,"{\"kind\":\"%s\",\"report\":{\"account\":%s,\"authentication\":\"%s\",\"retrieval\":{\"status\":\"%s\",\"device_ids\":[%s],\"error\":%s%s}}}",
      mode==21?"directory":connected?"connected_devices":"devices",account,
      mode==18?"unconfirmed":connected?"succeeded":"not_attempted",
      mode==9?"complete":(mode==12 || mode==16)?"failed":"partial",ids,
      mode==17?"null":mode==12?"{\"code\":\"MIJIA_REQUEST_FAILED\",\"message\":\"Fictional failure.\",\"retryable\":false}":"{\"code\":\"MIJIA_INVENTORY_PARTIAL\",\"message\":\"Fictional partial inventory.\",\"retryable\":false}",
      mode==19?",\"provider_id\":\"PRIVATE\"":"");
    char *value=response(projected); free(projected); free(ids); return value;
  }
#endif
  if (renewed) snprintf(data,sizeof(data),"{\"kind\":\"renewed\",\"account\":%s}",account);
  else snprintf(data,sizeof(data),"{\"kind\":\"%s\",\"report\":{\"account\":%s,\"authentication\":\"%s\",\"directory\":{\"status\":\"%s\",\"logical_entry_count\":%d,\"complete\":%s,\"issues\":[],\"error\":%s}}}",connected?"connected":"directory",account,connected?"succeeded":"not_attempted",mode==12?"failed":"partial",mode==10?-1:mode==12?0:3,mode==9?"true":"false",mode==12?"{\"code\":\"MIJIA_REQUEST_FAILED\",\"message\":\"Fictional directory failure.\",\"retryable\":false}":"null");
  return response(data);
}
char *homeverb_continue_modern_account_login(uint64_t handle,const char *metadata) {
  if (handle!=1 || !strstr(metadata,"\"challenge_id\":1") || strstr(metadata,"PRIVATE")) return failure();
#ifdef PUBLISH_DEVICES
  if (!strstr(metadata,"\"publish_devices\":true")) return failure();
#else
  if (strstr(metadata,"publish_devices")) return failure();
#endif
  continues++; stall(6); return result(false,true);
}
char *homeverb_modern_account_session(uint64_t handle,const char *metadata) {
  if (handle!=1 || !strstr(metadata,"00000000-0000-4000-8000-000000000001") || strstr(metadata,"PRIVATE")) return failure();
  sessions++; stall(7);
  if (strstr(metadata,"\"operation\":\"renew\"")) return result(true,false);
#ifdef PUBLISH_DEVICES
  if (!strstr(metadata,"\"operation\":\"refresh_devices\"") && !strstr(metadata,"\"operation\":\"recover_devices\"")) return failure();
#else
  if (!strstr(metadata,"\"operation\":\"refresh\"") && !strstr(metadata,"\"operation\":\"recover\"")) return failure();
#endif
  return result(false,false);
}
#endif

/* Test-only JNI adapter for the production C ABI. Never packaged in the AAR. */
#include <jni.h>
#include <fcntl.h>
#include <sys/file.h>
#include <unistd.h>
#include "homeverb.h"

JNIEXPORT jstring JNICALL Java_org_homeverb_AcceptanceRunner_request(
        JNIEnv *env, jclass type, jint action, jlong handle, jstring json) {
    (void)type;
    const char *input = (*env)->GetStringUTFChars(env, json, NULL);
    if (!input) return NULL;
    char *output = NULL;
    if (action == 0) output = homeverb_open(input);
    if (action == 1) output = homeverb_call((uint64_t)handle, input);
    if (action == 2) {
        uint8_t login[] = "fictional@example.invalid";
        uint8_t password[] = "fictional-test-password";
        output = homeverb_connect_account((uint64_t)handle, input,
            login, sizeof(login) - 1, password, sizeof(password) - 1);
        for (size_t i = 0; i < sizeof(login); ++i) login[i] = 0;
        for (size_t i = 0; i < sizeof(password); ++i) password[i] = 0;
    }
    (*env)->ReleaseStringUTFChars(env, json, input);
    jstring result = output ? (*env)->NewStringUTF(env, output) : NULL;
    homeverb_string_free(output);
    return result;
}

JNIEXPORT void JNICALL Java_org_homeverb_AcceptanceRunner_foreground(
        JNIEnv *env, jclass type, jlong handle, jboolean active) {
    (void)env;
    (void)type;
    homeverb_set_foreground((uint64_t)handle, active == JNI_TRUE);
}

JNIEXPORT void JNICALL Java_org_homeverb_AcceptanceRunner_close(
        JNIEnv *env, jclass type, jlong handle) {
    (void)env;
    (void)type;
    homeverb_close((uint64_t)handle);
}

JNIEXPORT jint JNICALL Java_org_homeverb_AcceptanceRunner_holdLock(
        JNIEnv *env, jclass type, jstring path) {
    (void)type;
    const char *name = (*env)->GetStringUTFChars(env, path, NULL);
    if (!name) return -1;
    int fd = open(name, O_RDWR);
    (*env)->ReleaseStringUTFChars(env, path, name);
    if (fd >= 0 && flock(fd, LOCK_EX | LOCK_NB) != 0) {
        close(fd);
        return -1;
    }
    return fd;
}

JNIEXPORT void JNICALL Java_org_homeverb_AcceptanceRunner_releaseLock(
        JNIEnv *env, jclass type, jint fd) {
    (void)env;
    (void)type;
    if (fd >= 0) close(fd);
}

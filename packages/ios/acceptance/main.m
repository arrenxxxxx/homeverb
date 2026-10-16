/* Isolated Simulator acceptance app. No provider/account or appliance requests. */
@import UIKit;
@import HomeVerb;
@import Security;
#include <unistd.h>

typedef struct { uint32_t code; uint32_t checks; } StoreResult;
extern StoreResult homeverb_test_ios_store(uint32_t phase, uint64_t high, uint64_t low);

static NSUInteger checks;
static NSString *stage = @"startup";

static void check(BOOL condition) {
    if (!condition) @throw [NSException exceptionWithName:@"ProbeFailure" reason:stage userInfo:nil];
    checks++;
}

static NSString *encode(id object) {
    NSData *data = [NSJSONSerialization dataWithJSONObject:object options:0 error:NULL];
    check(data != nil);
    return [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
}

static NSDictionary *decode(char *raw) {
    check(raw != NULL);
    NSData *data = [NSData dataWithBytes:raw length:strlen(raw)];
    homeverb_string_free(raw);
    id result = [NSJSONSerialization JSONObjectWithData:data options:0 error:NULL];
    check([result isKindOfClass:[NSDictionary class]]);
    return result;
}

static NSDictionary *call(uint64_t handle, NSDictionary *request) {
    return decode(homeverb_call(handle, [encode(@{
        @"schema_version": @"1.0", @"operation_id": NSUUID.UUID.UUIDString,
        @"timeout_ms": @10000, @"request": request
    }) UTF8String]));
}

static id query(uint64_t handle, NSDictionary *request) {
    NSDictionary *response = call(handle, request);
    check(response[@"error"] == NSNull.null);
    return response[@"data"];
}

/* Test-only metadata shape diagnostics; never request or report credential data. */
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
static NSDictionary *storeMetadata(NSString *runID) {
    NSDictionary *request = @{
        (__bridge id)kSecClass: (__bridge id)kSecClassGenericPassword,
        (__bridge id)kSecAttrService: [@"org.homeverb.credentials.v1." stringByAppendingString:runID.lowercaseString],
        (__bridge id)kSecAttrAccount: runID.lowercaseString,
        (__bridge id)kSecAttrAccessGroup: @"SYNTHETIC.org.homeverb.test",
        (__bridge id)kSecAttrSynchronizable: (__bridge id)kSecAttrSynchronizableAny,
        (__bridge id)kSecUseAuthenticationUI: (__bridge id)kSecUseAuthenticationUIFail,
        (__bridge id)kSecMatchLimit: @2,
        (__bridge id)kSecReturnAttributes: @YES, (__bridge id)kSecReturnPersistentRef: @YES
    };
    CFTypeRef raw = NULL;
    OSStatus status = SecItemCopyMatching((__bridge CFDictionaryRef)request, &raw);
    id result = CFBridgingRelease(raw);
    if (status != errSecSuccess || ![result isKindOfClass:NSArray.class] || [result count] != 1)
        return @{@"status": @(status), @"array": @([result isKindOfClass:NSArray.class])};
    NSDictionary *entry = result[0];
    if (![entry isKindOfClass:NSDictionary.class]) return @{@"dictionary": @NO};
    id sync = entry[(__bridge id)kSecAttrSynchronizable];
    id reference = entry[(__bridge id)kSecValuePersistentRef];
    return @{
        @"status": @(status), @"scope": @([entry[(__bridge id)kSecAttrAccessGroup] isEqual:@"SYNTHETIC.org.homeverb.test"]),
        @"passcode": @([entry[(__bridge id)kSecAttrAccessible] isEqual:(__bridge id)kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly]),
        @"sync_present": @(sync != nil), @"sync_boolean": @(sync && CFGetTypeID((__bridge CFTypeRef)sync) == CFBooleanGetTypeID()),
        @"sync_number": @([sync isKindOfClass:NSNumber.class]), @"sync_zero": @([sync isEqual:@0]),
        @"access_control": @(entry[(__bridge id)kSecAttrAccessControl] != nil),
        @"persistent_ref": @([reference isKindOfClass:NSData.class] && [reference length] > 0)
    };
}
#pragma clang diagnostic pop

static void coreFlow(NSString *documents, NSString *phase, NSString *runID) {
    stage = @"core_open";
    NSDictionary *opened = decode(homeverb_open([encode(@{
        @"schema_version": @"1.0", @"adapter": @"homeverb.mock", @"allow_mock_writes": @YES,
        @"data_dir": [documents stringByAppendingPathComponent:@"mock"]
    }) UTF8String]));
    check(opened[@"error"] == NSNull.null);
    uint64_t handle = [opened[@"data"][@"handle"] unsignedLongLongValue];
    check(handle != 0);
    @try {
        stage = @"core_api";
        check([query(handle, @{@"method": @"get_api_info"})[@"api_version"] isEqual:@"1.20.0"]);
        if ([phase isEqual:@"write"]) {
            stage = @"core_discover";
            NSArray *devices = query(handle, @{@"method": @"discover"});
            check(devices.count == 1 && [devices[0][@"id"] isEqual:@"dev-mock-ac"]);
            stage = @"core_execute";
            NSDictionary *receipt = call(handle, @{@"method": @"execute", @"command": @{
                @"schema_version": @"1.0", @"request_id": runID, @"device_id": @"dev-mock-ac",
                @"component_id": @"main", @"capability": @"power", @"action": @"set",
                @"arguments": @{@"state": @YES}, @"connection": @"auto",
                @"timeout_ms": @10000, @"wait_for": @"verified"
            }});
            check([receipt[@"status"] isEqual:@"verified"]);
            check([receipt[@"verification"][@"matches_target"] isEqual:@YES]);
            stage = @"core_read";
            NSDictionary *snapshot = query(handle, @{
                @"method": @"read_state", @"device": @"dev-mock-ac", @"component": @"main",
                @"connection": @"auto", @"cached": @NO
            });
            check([snapshot[@"observations"][@"power.state"][@"value"] isEqual:@YES]);
        } else {
            stage = @"core_persistence";
            NSArray *devices = query(handle, @{@"method": @"list_devices"});
            check(devices.count == 1 && [devices[0][@"id"] isEqual:@"dev-mock-ac"]);
            NSDictionary *receipt = query(handle, @{@"method": @"get_execution", @"request_id": runID});
            check([receipt[@"status"] isEqual:@"verified"] && [receipt[@"request_id"] isEqual:runID]);
            NSDictionary *snapshot = query(handle, @{
                @"method": @"read_state", @"device": @"dev-mock-ac", @"component": @"main",
                @"connection": @"auto", @"cached": @YES
            });
            check([snapshot[@"from_cache"] isEqual:@YES]);
            check([snapshot[@"observations"][@"power.state"][@"value"] isEqual:@YES]);
        }
        stage = @"core_lifecycle";
        homeverb_set_foreground(handle, false);
        check([call(handle, @{@"method": @"discover"})[@"error"][@"code"] isEqual:@"HOST_BACKGROUND"]);
        homeverb_set_foreground(handle, true);
        check([query(handle, @{@"method": @"list_devices"}) count] == 1);
    } @finally {
        homeverb_close(handle);
    }
    stage = @"core_closed";
    check([call(handle, @{@"method": @"list_devices"})[@"error"][@"code"] isEqual:@"HOST_CLOSED"]);
}

static void probe(void) {
    @autoreleasepool {
        NSArray<NSString *> *arguments = NSProcessInfo.processInfo.arguments;
        if (arguments.count != 3) return;
        NSString *phase = arguments[1], *runID = arguments[2];
        NSUUID *uuid = [[NSUUID alloc] initWithUUIDString:runID];
        if (!uuid || !([phase isEqual:@"write"] || [phase isEqual:@"read"])) return;
        NSString *documents = NSSearchPathForDirectoriesInDomains(NSDocumentDirectory, NSUserDomainMask, YES).firstObject;
        NSString *statePath = [documents stringByAppendingPathComponent:@"writer.plist"];
        NSMutableDictionary *report = [@{
            @"run_id": runID, @"phase": phase, @"pid": @(getpid()), @"status": @"failed",
            @"core_checks": @0, @"store_checks": @0, @"store_code": @0, @"writer_pid": @0
        } mutableCopy];
        @try {
            stage = @"process_identity";
            if ([phase isEqual:@"write"]) {
                check(![NSFileManager.defaultManager fileExistsAtPath:statePath]);
                check([@{@"run_id": runID, @"pid": @(getpid())} writeToFile:statePath atomically:YES]);
                report[@"writer_pid"] = @(getpid());
            } else {
                NSDictionary *writer = [NSDictionary dictionaryWithContentsOfFile:statePath];
                check([writer[@"run_id"] isEqual:runID] && [writer[@"pid"] intValue] > 0);
                check([writer[@"pid"] intValue] != getpid());
                report[@"writer_pid"] = writer[@"pid"];
            }
            coreFlow(documents, phase, runID);
            report[@"core_checks"] = @(checks);
            stage = @"keychain";
            uuid_t bytes;
            [uuid getUUIDBytes:bytes];
            uint64_t high = 0, low = 0;
            for (int i = 0; i < 8; i++) { high = (high << 8) | bytes[i]; low = (low << 8) | bytes[i + 8]; }
            StoreResult result = homeverb_test_ios_store([phase isEqual:@"write"] ? 1 : 2, high, low);
            report[@"store_code"] = @(result.code);
            report[@"store_checks"] = @(result.checks);
            if (result.code != 0) report[@"metadata_shape"] = storeMetadata(runID);
            check(result.code == 0 && result.checks == ([phase isEqual:@"write"] ? 10 : 4));
            report[@"status"] = @"passed";
            stage = @"complete";
        } @catch (NSException *exception) {
            (void)exception; /* Never report native responses, exception text or credential bytes. */
        }
        report[@"stage"] = stage;
        NSData *data = [NSJSONSerialization dataWithJSONObject:report options:0 error:NULL];
        [data writeToFile:[documents stringByAppendingPathComponent:[phase stringByAppendingString:@".json"]]
                 options:NSDataWritingAtomic error:NULL];
    }
}

@interface AcceptanceApp : UIResponder <UIApplicationDelegate>
@property(strong, nonatomic) UIWindow *window;
@end
@implementation AcceptanceApp
- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)options {
    (void)application; (void)options;
    self.window = [[UIWindow alloc] initWithFrame:UIScreen.mainScreen.bounds];
    self.window.rootViewController = [UIViewController new];
    [self.window makeKeyAndVisible];
    dispatch_async(dispatch_get_global_queue(QOS_CLASS_USER_INITIATED, 0), ^{ probe(); });
    return YES;
}
@end

int main(int argc, char **argv) {
    @autoreleasepool { return UIApplicationMain(argc, argv, nil, NSStringFromClass(AcceptanceApp.class)); }
}

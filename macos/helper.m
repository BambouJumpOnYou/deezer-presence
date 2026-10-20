// Prints Deezer's now playing info as one JSON line per second.
//
// MediaRemote is restricted to Apple-signed binaries since macOS 15.4, so this library is loaded into
// /usr/bin/perl, a technique borrowed from mediaremote-adapter (https://github.com/ungive/mediaremote-adapter).
// Unlike the system "Now Playing", it asks for Deezer's player specifically, so other media can't hide it.

#import <Foundation/Foundation.h>
#import <dispatch/dispatch.h>
#include <math.h>
#include <stdio.h>

static NSString *const kDeezerBundleID = @"com.deezer.deezer-desktop";

@protocol MRClient
- (NSString *)bundleIdentifier;
- (NSString *)parentApplicationBundleIdentifier;
@end

typedef void (*GetClients)(dispatch_queue_t, void (^)(NSArray *));
// Reverse-engineered: builds an MRPlayerPath from (client, origin) and forwards to GetNowPlayingInfoForPlayer.
typedef void (*GetInfoForClient)(id client, id origin, uintptr_t unused, dispatch_queue_t, void (^)(NSDictionary *));
typedef id (*GetLocalOrigin)(void);

static id deezerClient(NSArray *clients) {
    for (id<MRClient> client in clients) {
        if ([[client bundleIdentifier] isEqualToString:kDeezerBundleID] ||
            [[client parentApplicationBundleIdentifier] isEqualToString:kDeezerBundleID]) {
            return client;
        }
    }
    return nil;
}

static NSDictionary *payload(NSDictionary *info) {
    NSMutableDictionary *out = [NSMutableDictionary dictionary];
    NSDictionary *keys = @{
        @"kMRMediaRemoteNowPlayingInfoTitle" : @"title",
        @"kMRMediaRemoteNowPlayingInfoArtist" : @"artist",
        @"kMRMediaRemoteNowPlayingInfoAlbum" : @"album",
        @"kMRMediaRemoteNowPlayingInfoDuration" : @"duration",
        @"kMRMediaRemoteNowPlayingInfoElapsedTime" : @"elapsed",
        @"kMRMediaRemoteNowPlayingInfoPlaybackRate" : @"rate",
    };
    for (NSString *key in keys) {
        id value = info[key];
        if ([value isKindOfClass:[NSNumber class]] && !isfinite([value doubleValue])) {
            continue;
        }
        if ([value isKindOfClass:[NSString class]] || [value isKindOfClass:[NSNumber class]]) {
            out[keys[key]] = value;
        }
    }
    NSDate *timestamp = info[@"kMRMediaRemoteNowPlayingInfoTimestamp"];
    if ([timestamp isKindOfClass:[NSDate class]]) {
        out[@"timestamp"] = @([timestamp timeIntervalSince1970]);
    }
    return out;
}

static void printLine(NSDictionary *object) {
    NSData *json = [NSJSONSerialization dataWithJSONObject:object options:0 error:nil];
    fwrite(json.bytes, 1, json.length, stdout);
    fputc('\n', stdout);
    fflush(stdout);
}

void deezer_presence_poll(void) {
    CFURLRef url = CFURLCreateWithFileSystemPath(
        kCFAllocatorDefault, CFSTR("/System/Library/PrivateFrameworks/MediaRemote.framework"),
        kCFURLPOSIXPathStyle, true);
    CFBundleRef bundle = CFBundleCreate(kCFAllocatorDefault, url);
    CFRelease(url);
    GetClients getClients =
        (GetClients)CFBundleGetFunctionPointerForName(bundle, CFSTR("MRMediaRemoteGetNowPlayingClients"));
    GetInfoForClient getInfo =
        (GetInfoForClient)CFBundleGetFunctionPointerForName(bundle, CFSTR("MRMediaRemoteGetNowPlayingInfoForClient"));
    GetLocalOrigin getLocalOrigin =
        (GetLocalOrigin)CFBundleGetFunctionPointerForName(bundle, CFSTR("MRMediaRemoteGetLocalOrigin"));
    if (!getClients || !getInfo || !getLocalOrigin) {
        fprintf(stderr, "MediaRemote functions not found\n");
        exit(1);
    }

    id origin = getLocalOrigin();
    dispatch_queue_t queue = dispatch_queue_create("deezer-presence", DISPATCH_QUEUE_SERIAL);
    while (true) {
        @autoreleasepool {
            __block NSDictionary *result = @{};
            dispatch_semaphore_t done = dispatch_semaphore_create(0);
            getClients(queue, ^(NSArray *clients) {
              id client = deezerClient(clients);
              if (!client) {
                  dispatch_semaphore_signal(done);
                  return;
              }
              getInfo(client, origin, 0, queue, ^(NSDictionary *info) {
                result = payload(info);
                dispatch_semaphore_signal(done);
              });
            });
            dispatch_semaphore_wait(done, dispatch_time(DISPATCH_TIME_NOW, 2 * NSEC_PER_SEC));
            printLine(result);
        }
        sleep(1);
    }
}

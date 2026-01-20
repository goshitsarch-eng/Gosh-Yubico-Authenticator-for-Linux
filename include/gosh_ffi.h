#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct GoshClient GoshClient;

typedef struct GoshVersion {
    uint8_t major;
    uint8_t minor;
    uint8_t patch;
} GoshVersion;

typedef struct GoshCredential {
    const uint8_t* id_ptr;
    size_t id_len;
    const char* issuer;
    const char* account;
    const char* code;
    uint8_t oath_type;
    uint8_t algorithm;
    uint8_t digits;
    uint8_t touch_required;
    uint32_t period;
} GoshCredential;

typedef struct GoshEvent {
    uint32_t event_type;
    GoshVersion version;
    uint8_t device_id[8];
    const char* message;
    const GoshCredential* credentials;
    size_t credentials_len;
    const uint8_t* id_ptr;
    size_t id_len;
    const char* code;
    uint8_t digits;
} GoshEvent;

typedef void (*GoshEventCallback)(const GoshEvent* event, void* user_data);

void gosh_init_logging(void);

GoshClient* gosh_client_new(void);
void gosh_client_free(GoshClient* client);

void gosh_client_set_callback(GoshClient* client, GoshEventCallback callback, void* user_data);
void gosh_client_connect(GoshClient* client);
void gosh_client_refresh(GoshClient* client);
void gosh_client_authenticate(GoshClient* client, const char* password);
void gosh_client_calculate(GoshClient* client, const uint8_t* id_ptr, size_t id_len);
void gosh_client_delete_credential(GoshClient* client, const uint8_t* id_ptr, size_t id_len);

bool gosh_client_add_credential(
    GoshClient* client,
    const char* issuer,
    const char* account,
    const char* secret,
    uint8_t oath_type,
    uint8_t algorithm,
    uint8_t digits,
    uint8_t require_touch,
    uint32_t initial_counter,
    uint8_t has_initial_counter);

char* gosh_client_last_error_take(GoshClient* client);
void gosh_string_free(char* value);
void gosh_event_free(GoshEvent* event);

#ifdef __cplusplus
}
#endif

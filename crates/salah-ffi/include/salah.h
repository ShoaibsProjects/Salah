#ifndef SALAH_NATIVE_H
#define SALAH_NATIVE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ABI 1. Request is a borrowed UTF-8 buffer of at most 8192 bytes.
 * The caller must keep valid memory for the call, never concurrently mutate it.
 * Operations: 0 inventory (no request), 1 zone lookup, 2 local clock,
 * 3 schedule with explicit selection (v2). No sensors, clock or network read.
 * Response is opaque and owned by Rust. Copy data before freeing exactly once.
 * Never free concurrently with a getter call or a read of its returned bytes.
 * Null response means an unrecoverable boundary failure. Ordinary input errors
 * are JSON error responses. Abort/OOM and invalid foreign pointers cannot be
 * recovered into JSON. Never pass a fabricated/freed response pointer. */
typedef struct SalahResponse SalahResponse;
uint32_t salah_native_abi_version(void);
SalahResponse *salah_execute(uint32_t operation, const uint8_t *request, size_t length);
const uint8_t *salah_response_data(const SalahResponse *response);
size_t salah_response_length(const SalahResponse *response);
void salah_response_free(SalahResponse *response);

#ifdef __cplusplus
}
#endif
#endif

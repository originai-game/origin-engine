/* Origin Engine C ABI host (crates/og-host-c): the symbol table a native
 * shell registers with its JavaScript engine. An answer's memory is the
 * host's until the next entry is called on the thread. */
#ifndef OG_HOST_H
#define OG_HOST_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum {
  OG_UNDEFINED = 0,
  OG_BOOL = 1,
  OG_NUMBER = 2,
  OG_STRING = 3,    /* UTF-8, len bytes */
  OG_BYTES = 4,     /* a Uint8Array, len bytes */
  OG_U32_ARRAY = 5, /* a Uint32Array, len words */
  OG_VIEWS = 6,     /* len OgColumn */
  OG_FUNCTION = 7,  /* ptr: an OgDispatcher the shell keeps alive */
  OG_SHARED = 8,    /* the shared buffer, len bytes */
};

typedef struct OgValue {
  uint32_t tag;
  uint32_t len;
  double num;
  const void *ptr;
} OgValue;

typedef struct OgColumn {
  uint8_t *ptr;
  size_t len;
  bool writable;
} OgColumn;

typedef struct OgDispatcher {
  void *ctx;
  void (*call)(void *ctx, uint32_t kind, uint32_t engine, uint32_t id,
               uint32_t query, double tick, double dt, double elapsed,
               const uint8_t *payload, size_t payload_len,
               const uint8_t **answer, size_t *answer_len);
  void (*invalidate)(void *ctx, bool moving);
} OgDispatcher;

typedef struct OgSymbol {
  const char *name;
  size_t arity;
  void (*call)(const OgValue *args, OgValue *out);
} OgSymbol;

const OgSymbol *og_c_symbols(size_t *count);

/* The desktop window (feature `window`, src/window.rs). `open` answers wire
 * JSON, valid until the next call; `pump` is false once the window closed;
 * `capture` writes the last presented frame as a PNG (opened with keep);
 * `close` drops the renderer and the window, after the engine. */
const char *og_c_window_open(uint32_t engine, uint32_t width, uint32_t height,
                             const char *title, bool keep);
bool og_c_window_pump(void);
bool og_c_window_capture(const char *path);
void og_c_window_close(void);

#ifdef __cplusplus
}
#endif

#endif

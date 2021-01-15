/* Generated with cbindgen:0.16.0 */

/* DO NOT MODIFY THIS MANUALLY */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum KimeInputResultType {
  Bypass,
  Consume,
  ClearPreedit,
  Preedit,
  Commit,
  CommitBypass,
  CommitPreedit,
  CommitCommit,
} KimeInputResultType;

typedef struct KimeConfig KimeConfig;

typedef struct KimeInputEngine KimeInputEngine;

typedef struct KimeInputResult {
  enum KimeInputResultType ty;
  uint32_t char1;
  uint32_t char2;
} KimeInputResult;

struct KimeInputEngine *kime_engine_new(void);

void kime_engine_delete(struct KimeInputEngine *engine);

uint32_t kime_engine_preedit_char(struct KimeInputEngine *engine);

uint32_t kime_engine_reset(struct KimeInputEngine *engine);

struct KimeInputResult kime_engine_press_key(struct KimeInputEngine *engine,
                                             const struct KimeConfig *config,
                                             uint16_t hardware_code,
                                             uint32_t state);

struct KimeConfig *kime_config_load(void);

void kime_config_delete(struct KimeConfig *config);

uint32_t kime_config_gtk_commit_english(struct KimeConfig *config);

void kime_config_xim_preedit_font(struct KimeConfig *config, const uint8_t **name, uintptr_t *len);

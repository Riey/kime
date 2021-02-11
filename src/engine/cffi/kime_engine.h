#pragma once

/* Generated with cbindgen:0.17.0 */

/* DO NOT MODIFY THIS MANUALLY */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

enum KimeInputResultType {
  Bypass,
  Consume,
  Preedit,
  Commit,
  CommitBypass,
  CommitPreedit,
  CommitCommit,
};
typedef uint16_t KimeInputResultType;

typedef struct KimeConfig KimeConfig;

typedef struct KimeInputEngine KimeInputEngine;

typedef struct KimeInputResult {
  KimeInputResultType ty;
  bool hangul_changed;
  uint32_t char1;
  uint32_t char2;
} KimeInputResult;

typedef uint32_t KimeModifierState;
#define KimeModifierState_CONTROL (uint32_t)1
#define KimeModifierState_SUPER (uint32_t)2
#define KimeModifierState_SHIFT (uint32_t)4
#define KimeModifierState_ALT (uint32_t)8

/**
 * Create new engine
 */
struct KimeInputEngine *kime_engine_new(void);

/**
 * Set hangul enable state
 */
void kime_engine_set_hangul_enable(struct KimeInputEngine *engine, bool mode);

/**
 * Delete engine
 *
 * # Safety
 *
 * engine must be created by `kime_engine_new` and never call delete more than once
 */
void kime_engine_delete(struct KimeInputEngine *engine);

/**
 * Update hangul state
 */
void kime_engine_update_hangul_state(struct KimeInputEngine *engine);

/**
 * Get preedit_char of engine
 *
 * ## Return
 *
 * valid ucs4 char NULL to represent empty
 */
uint32_t kime_engine_preedit_char(const struct KimeInputEngine *engine);

/**
 * Reset preedit state then returm commit char
 *
 * ## Return
 *
 * valid ucs4 char NULL to represent empty
 */
uint32_t kime_engine_reset(struct KimeInputEngine *engine);

/**
 * Press key when modifier state
 *
 * ## Return
 *
 * input result
 */
struct KimeInputResult kime_engine_press_key(struct KimeInputEngine *engine,
                                             const struct KimeConfig *config,
                                             uint16_t hardware_code,
                                             KimeModifierState state);

/**
 * Load config from local file
 */
struct KimeConfig *kime_config_load(void);

/**
 * Create default config note that this function will not read config file
 */
struct KimeConfig *kime_config_default(void);

/**
 * Delete config
 */
void kime_config_delete(struct KimeConfig *config);

/**
 * Get xim_preedit_font config
 * name only valid while config is live
 *
 * ## Return
 *
 * utf-8 string when len
 */
void kime_config_xim_preedit_font(const struct KimeConfig *config,
                                  const uint8_t **name,
                                  uintptr_t *len,
                                  double *font_size);

#pragma once

/* Generated with cbindgen:0.17.0 */

/* DO NOT MODIFY THIS MANUALLY */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define KimeKIME_API_VERSION 2

typedef struct KimeConfig KimeConfig;

typedef struct KimeInputEngine KimeInputEngine;

typedef struct KimeRustStr {
  const uint8_t *ptr;
  uintptr_t len;
} KimeRustStr;

typedef uint32_t KimeInputResult;
#define KimeInputResult_CONSUMED (uint32_t)1
#define KimeInputResult_LANGUAGE_CHANGED (uint32_t)2
#define KimeInputResult_HAS_PREEDIT (uint32_t)4
#define KimeInputResult_NEED_RESET (uint32_t)8
#define KimeInputResult_NEED_FLUSH (uint32_t)16

typedef uint32_t KimeModifierState;
#define KimeModifierState_CONTROL (uint32_t)1
#define KimeModifierState_SUPER (uint32_t)2
#define KimeModifierState_SHIFT (uint32_t)4
#define KimeModifierState_ALT (uint32_t)8

typedef struct KimeXimPreeditFont {
  struct KimeRustStr name;
  double size;
} KimeXimPreeditFont;

/**
 * Return API version
 */
uintptr_t kime_api_version(void);

/**
 * Create new engine
 */
struct KimeInputEngine *kime_engine_new(const struct KimeConfig *config);

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
 * Get commit string of engine
 *
 * ## Return
 *
 * valid utf8 string
 */
struct KimeRustStr kime_engine_commit_str(struct KimeInputEngine *engine);

/**
 * Get preedit string of engine
 *
 * ## Return
 *
 * valid utf8 string
 */
struct KimeRustStr kime_engine_preedit_str(struct KimeInputEngine *engine);

/**
 * Flush commit_str
 */
void kime_engine_flush(struct KimeInputEngine *engine);

/**
 * Clear preedit state and append to commit_str
 */
void kime_engine_clear_preedit(struct KimeInputEngine *engine);

/**
 * Reset preedit state then returm commit char
 */
void kime_engine_reset(struct KimeInputEngine *engine);

/**
 * Press key when modifier state
 *
 * ## Return
 *
 * input result
 */
KimeInputResult kime_engine_press_key(struct KimeInputEngine *engine,
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
struct KimeXimPreeditFont kime_config_xim_preedit_font(const struct KimeConfig *config);

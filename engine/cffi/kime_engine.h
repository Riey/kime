#pragma once

/* Generated with cbindgen:0.16.0 */

/* DO NOT MODIFY THIS MANUALLY */

#include <stdint.h>
#include <stddef.h>

typedef enum InputResultType {
  Bypass,
  ToggleHangul,
  ClearPreedit,
  Preedit,
  Commit,
  CommitBypass,
  CommitPreedit,
  CommitCommit,
} InputResultType;

typedef struct Config Config;

typedef struct InputEngine InputEngine;

typedef struct InputResult {
  enum InputResultType ty;
  uint32_t char1;
  uint32_t char2;
} InputResult;

typedef uint32_t ModifierState;
#define ModifierState_CONTROL (uint32_t)1
#define ModifierState_SUPER (uint32_t)2
#define ModifierState_SHIFT (uint32_t)4

/**
 * Create new engine
 */
struct InputEngine *kime_engine_new(void);

/**
 * Delete engine
 */
void kime_engine_delete(struct InputEngine *engine);

/**
 * Get preedit_char of engine
 *
 * ## Return
 *
 * valid ucs4 char NULL to represent empty
 */
uint32_t kime_engine_preedit_char(const struct InputEngine *engine);

/**
 * Reset preedit state then returm commit char
 *
 * ## Return
 *
 * valid ucs4 char NULL to represent empty
 */
uint32_t kime_engine_reset(struct InputEngine *engine);

/**
 * Press key when modifier state
 *
 * ## Return
 *
 * input result
 */
struct InputResult kime_engine_press_key(struct InputEngine *engine,
                                         const struct Config *config,
                                         uint16_t hardware_code,
                                         ModifierState state);

/**
 * Load config from local file
 */
struct Config *kime_config_load(void);

/**
 * Delete config
 */
void kime_config_delete(struct Config *config);

/**
 * Get xim_preedit_font config
 * name only valid while config is live
 *
 * ## Return
 *
 * utf-8 string when len
 */
void kime_config_xim_preedit_font(const struct Config *config,
                                  const uint8_t **name,
                                  uintptr_t *len,
                                  double *font_size);

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct Player Player;

typedef struct {
  const uint8_t *data_ptr;
  size_t data_len;
  double pts;
  double dts;
  double duration;
  bool is_key;
  bool is_last;
} FFISample;

typedef struct {
  const uint8_t *sps_ptr;
  size_t sps_len;
  const uint8_t *pps_ptr;
  size_t pps_len;
  uint32_t width;
  uint32_t height;
  uint8_t nal_length_size;
} FFICodecConfig;

Player *player_new(const char *base_url);
void player_free(Player *player);

void player_start_loading(Player *player);
void player_play(Player *player, double now);
void player_pause(Player *player, double now);
void player_seek(Player *player, double media, double now);
void player_set_playback_rate(Player *player, double rate, double now);

bool player_is_playing(const Player *player);
double player_get_media_time(const Player *player, double now);

FFISample *player_pop_sample(Player *player);
void sample_free(FFISample *sample);

FFICodecConfig *get_codec_config(const char *base_url);
void free_codec_config(FFICodecConfig *config);

const char *get_manifest_json(const char *base_url);
void free_string(const char *string);

void register_log_callback(void (*callback)(const char *));
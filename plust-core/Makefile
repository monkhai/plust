UNAME := $(shell uname)
ifeq ($(UNAME), Darwin)
    NDK_HOST := darwin-x86_64
    ANDROID_NDK_HOME ?= $(HOME)/Library/Android/sdk/ndk/29.0.14206865
else
    NDK_HOST := linux-x86_64
    ANDROID_NDK_HOME ?= $(HOME)/Android/Sdk/ndk/29.0.14206865
endif

NDK_BIN := $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(NDK_HOST)/bin

android:
	CC_aarch64_linux_android=$(NDK_BIN)/aarch64-linux-android21-clang \
	AR_aarch64_linux_android=$(NDK_BIN)/llvm-ar \
	CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$(NDK_BIN)/aarch64-linux-android21-clang \
	cargo build --release --target aarch64-linux-android

ios:
	cargo build --release --target aarch64-apple-ios

ios-sim:
	cargo build --release --target aarch64-apple-ios-sim

wasm:
	wasm-pack build --target web --out-dir ../plust-web/pkg

all: android ios ios-sim wasm

.PHONY: android ios ios-sim wasm all

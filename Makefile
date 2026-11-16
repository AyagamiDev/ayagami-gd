BUILD_NAME=libayagami_gd
ifneq (,$(findstring darwin,$(target)))
	EXT=dylib
	ARCH=aarch64
else ifneq (,$(findstring windows,$(target)))
	EXT=dll
	ARCH=x86_64
	BUILD_NAME=ayagami_gd
else
	EXT=so
	ARCH=x86_64
endif


debug:
	cargo build --target $(target)
	cp target/${target}/debug/$(BUILD_NAME).$(EXT) addons/ayagami/lib/libayagami_gd.debug.$(ARCH).$(EXT)

release:
	cargo build --release --target $(target)
	cp target/${target}/release/$(BUILD_NAME).$(EXT) addons/ayagami/lib/libayagami_gd.release.$(ARCH).$(EXT)

package: debug | release

PLATFORM=$(shell uname -o)
ifeq ($(PLATFORM),Darwin)
	EXT=dylib
else
	EXT=so
endif

ARCH=$(shell uname -m)

debug:
	cargo build --target $(target)
	cp target/${target}/debug/libayagami_gd.$(EXT) addons/ayagami/lib/libayagami_gd.debug.$(ARCH).$(EXT)

release:
	cargo build --release --target $(target)
	cp target/${target}/release/libayagami_gd.$(EXT) addons/ayagami/lib/libayagami_gd.release.$(ARCH).$(EXT)

package: debug | release

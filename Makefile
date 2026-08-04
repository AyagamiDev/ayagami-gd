debug:
	cargo build --target $(target)
	cp target/${target}/debug/libayagami_gd.so addons/ayagami/lib/libayagami_gd.debug.so

release:
	cargo build --release --target $(target)
	cp target/${target}/release/libayagami_gd.so addons/ayagami/lib/libayagami_gd.release.so

package: debug | release

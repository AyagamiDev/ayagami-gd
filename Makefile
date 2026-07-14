build:
	cargo build
	cp target/debug/libayagami_gd.so addons/ayagami/lib/libayagami_gd.debug.so

release:
	cargo build --release
	cp target/release/libayagami_gd.so addons/ayagami/lib/libayagami_gd.release.so

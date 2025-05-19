SOURCES := book.toml src

NAME := topiary
BUILD_DIR := man
BUILD_PREFIX := $(BUILD_DIR)/$(NAME)

PREFIX := /usr/share/man

MDBOOK_MANMUNGE := cargo run --package mdbook-manmunge --locked --

all: $(BUILD_PREFIX).1.gz

$(BUILD_PREFIX).1.gz: $(BUILD_PREFIX).man
	$(MDBOOK_MANMUNGE) post-process < "$<" \
	| gzip --stdout \
	> "$@"

$(BUILD_PREFIX).man: $(SOURCES)
	mdbook build

install: $(PREFIX)/man1/$(NAME).1.gz

$(PREFIX)/man1/$(NAME).1.gz: $(BUILD_PREFIX).1.gz
	install \
	  --owner root \
	  --group root \
	  --mode 0644 \
	  --no-target-directory \
	  "$<" \
	  "$@"

uninstall:
	rm -rf "$(PREFIX)/man1/$(NAME).1.gz"

clean:
	rm -rf "$(BUILD_DIR)"

.PHONEY: all install uninstall clean

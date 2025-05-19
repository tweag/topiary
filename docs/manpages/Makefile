SOURCES := book.toml src

BUILD_DIR := man
PREFIX := $(BUILD_DIR)/topiary

MDBOOK_MANMUNGE := cargo run --package mdbook-manmunge --locked --

all: $(PREFIX).1.gz

$(PREFIX).1.gz: $(PREFIX).man
	$(MDBOOK_MANMUNGE) post-process < $< \
	| gzip --stdout \
	> $@

$(PREFIX).man: $(SOURCES)
	mdbook build

clean:
	rm -rf $(BUILD_DIR)

.PHONEY: all clean

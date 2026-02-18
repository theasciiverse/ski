#
# makefile for `ski'
#
# SPDX-FileCopyrightText: (C) Eric S. Raymond <esr@thyrsus.com>
# SPDX-License-Identifier: BSD-2-Clause
#

PREFIX      ?= /usr/local
BINDIR      ?= $(PREFIX)/bin
DATADIR     ?= $(PREFIX)/share
MANDIR      ?= $(DATADIR)/man

VERS=$(shell sed <ski -n -e '/version *= *"\(.*\)"/s//\1/p')

# Rules

# Note: to suppress the footers with timestamps being generated in HTML,
# we use "-a nofooter".
# To debug asciidoc problems, you may need to run "xmllint --nonet --noout --valid"
# on the intermediate XML that throws an error.
.SUFFIXES: .html .adoc .6

.adoc.6:
	asciidoctor -D. -a nofooter -b manpage $<
.adoc.html:
	asciidoctor -D. -a nofooter -a webfonts! $<

.PHONY: all clean pylint spellcheck
.PHONY: install uninstall version dist release refresh

# Build

all: ski.6

clean:
	rm -f *~ *.6 *.html

# Validate

pylint:
	@pylint --score=n ski

spellcheck:
	@spellcheck ski.adoc

# Intall/uninstall

install: ski.6 uninstall
	install -m 0755 -d $(DESTDIR)$(BINDIR)
	install -m 0755 -d $(DESTDIR)$(MANDIR)/man6
	install -m 0755 -d $(DESTDIR)$(DATADIR)/applications/
	install -m 0755 -d $(DESTDIR)$(DATADIR)/pixmaps/
	install -m 0755 -d $(DESTDIR)$(DATADIR)/appdata
	install -m 0755 ski $(DESTDIR)$(BINDIR)/
	install -m 0644 ski.6 $(DESTDIR)$(MANDIR)/man6/
	install -m 0644 ski.desktop $(DESTDIR)$(DATADIR)/applications
	install -m 0644 ski.png $(DESTDIR)$(DATADIR)/pixmaps/
	install -m 0644 ski.adoc $(DESTDIR)$(DATADIR)/appdata/

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/ski $(MANDIR)/man6/ski.6
	rm -f $(DESTDIR)$(DATADIR)/applications/ski.desktop
	rm -f $(DESTDIR)$(DATADIR)/pixmaps/ski.png
	rm -f $(DESTDIR)$(DATADIR)/appdata/ski.adoc

# Export

SOURCES = README.adoc COPYING NEWS ski ski.adoc ski.6 Makefile \
	control ski.png ski.desktop

version:
	@echo $(VERSION)

sng-$(VERSION).tar.gz: $(SOURCES)
	mkdir sng-$(VERSION)
	cp -r $(SOURCES) sng-$(VERSION)
	tar -czf sng-$(VERSION).tar.gz sng-$(VERSION)
	rm -fr sng-$(VERSION)
	ls -l sng-$(VERSION).tar.gz

dist: ski-$(VERSION).tar.gz

NEWSVERSION=$(shell sed -n <NEWS '/^[0-9]/s/:.*//p' | head -1)

release: ski-$(VERSION).tar.gz ski.html
	@[ $(VERSION) = $(NEWSVERSION) ] || { echo "Version mismatch!"; exit 1; }
	shipper version=$(VERSION) | sh -e -x

refresh: ski.html
	@[ $(VERSION) = $(NEWSVERSION) ] || { echo "Version mismatch!"; exit 1; }
	shipper -N -w version=$(VERSION) | sh -e -x

# end

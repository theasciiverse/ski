#
# makefile for `ski'
#
# SPDX-FileCopyrightText: (C) Eric S. Raymond <esr@thyrsus.com>
# SPDX-License-Identifier: BSD-2-Clause
#
VERS=$(shell sed <ski -n -e '/version *= *"\(.*\)"/s//\1/p')

SOURCES = README COPYING NEWS ski ski.adoc ski.6 Makefile \
	control ski.png ski.desktop

all: ski.6

# Note: to suppress the footers with timestamps being generated in HTML,
# we use "-a nofooter".
# To debug asciidoc problems, you may need to run "xmllint --nonet --noout --valid"
# on the intermediate XML that throws an error.
.SUFFIXES: .html .adoc .6

.adoc.6:
	asciidoctor -D. -a nofooter -b manpage $<
.adoc.html:
	asciidoctor -D. -a nofooter -a webfonts! $<

clean:
	rm -f *~ *.6 *.html *.rpm *.lsm MANIFEST

install: ski.6 uninstall
	install -m 0755 -d $(DESTDIR)/usr/bin
	install -m 0755 -d $(DESTDIR)/usr/share/man/man6
	install -m 0755 -d $(DESTDIR)//usr/share/applications/
	install -m 0755 -d $(DESTDIR)/usr/share/pixmaps/
	install -m 0755 -d $(DESTDIR)/usr/share/appdata
	install -m 0755 ski $(DESTDIR)/usr/bin/
	install -m 0644 ski.6 $(DESTDIR)/usr/share/man/man6/
	install -m 0644 ski.desktop $(DESTDIR)/usr/share/applications
	install -m 0644 ski.png $(DESTDIR)/usr/share/pixmaps/
	install -m 0644 ski.adoc $(DESTDIR)/usr/share/appdata/

uninstall:
	rm -f /usr/bin/ski /usr/share/man/man6/ski.6
	rm -f /usr/share/applications/ski.desktop
	rm -f /usr/share/pixmaps/ski.png
	rm -f /usr/share/appdata/ski.adoc

pylint:
	@pylint --score=n ski

spellcheck:
	@spellcheck ski.adoc

version:
	@echo $(VERS)

ski-$(VERS).tar.gz: $(SOURCES)
	@ls $(SOURCES) | sed s:^:ski-$(VERS)/: >MANIFEST
	@(cd ..; ln -s ski ski-$(VERS))
	(cd ..; tar -czf ski/ski-$(VERS).tar.gz `cat ski/MANIFEST`)
	@ls -l ski-$(VERS).tar.gz
	@(cd ..; rm ski-$(VERS))

dist: ski-$(VERS).tar.gz

NEWSVERSION=$(shell sed -n <NEWS '/^[0-9]/s/:.*//p' | head -1)

release: ski-$(VERS).tar.gz ski.html
	@[ $(VERS) = $(NEWSVERSION) ] || { echo "Version mismatch!"; exit 1; }
	shipper version=$(VERS) | sh -e -x

refresh: ski.html
	@[ $(VERS) = $(NEWSVERSION) ] || { echo "Version mismatch!"; exit 1; }
	shipper -N -w version=$(VERS) | sh -e -x

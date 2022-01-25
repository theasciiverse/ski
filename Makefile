#
# makefile for `ski'
#
VERS=$(shell sed <ski -n -e '/version *= *"\(.*\)"/s//\1/p')

SOURCES = README COPYING NEWS ski ski.xml ski.6 Makefile \
	control ski.png ski.desktop

all: ski.6

ski.6: ski.xml
	xmlto man ski.xml

ski.html: ski.xml
	xmlto html-nochunks ski.xml

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
	install -m 0644 ski.xml $(DESTDIR)/usr/share/appdata/

uninstall:
	rm -f /usr/bin/ski /usr/share/man/man6/ski.6
	rm -f /usr/share/applications/ski.desktop
	rm -f /usr/share/pixmaps/ski.png
	rm -f /usr/share/appdata/ski.xml

pychecker:
	@ln -f ski ski.py
	@-pychecker --only --limit 50 ski.py
	@rm -f ski.py*

pylint:
	@pylint --score=n ski

version:
	@echo $(VERS)

ski-$(VERS).tar.gz: $(SOURCES)
	@ls $(SOURCES) | sed s:^:ski-$(VERS)/: >MANIFEST
	@(cd ..; ln -s ski ski-$(VERS))
	(cd ..; tar -czf ski/ski-$(VERS).tar.gz `cat ski/MANIFEST`)
	@ls -l ski-$(VERS).tar.gz
	@(cd ..; rm ski-$(VERS))

dist: ski-$(VERS).tar.gz

release: ski-$(VERS).tar.gz ski.html
	shipper version=$(VERS) | sh -e -x

refresh: ski.html
	shipper -N -w version=$(VERS) | sh -e -x

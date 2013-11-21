#
# makefile for `ski'
#
VERS=$(shell sed <ski -n -e '/version="\(.*\)"/s//\1/p')

SOURCES = README COPYING NEWS ski ski.xml ski.6 Makefile control ski.png

all: ski.6

ski.6: ski.xml
	xmlto man ski.xml

ski.html: ski.xml
	xmlto html-nochunks ski.xml

clean:
	rm -f *~ *.6 *.html *.rpm *.lsm MANIFEST

pychecker:
	@ln -f ski ski.py
	@-pychecker --only --limit 50 ski.py
	@rm -f ski.py*

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

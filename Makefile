#
# makefile for `ski'
#
VERS=$(shell sed <ski.spec -n -e '/Version: \(.*\)/s//\1/p')

SOURCES = README COPYING ski ski.xml ski.6 Makefile ski.spec

all: ski.6

ski.6: ski.xml
	xmlto man ski.xml

ski.html: ski.xml
	xmlto html-nochunks ski.xml

clean:
	rm -f *~ *.6 ski.html

ski-$(VERS).tar.gz: $(SOURCES)
	@ls $(SOURCES) | sed s:^:ski-$(VERS)/: >MANIFEST
	@(cd ..; ln -s ski ski-$(VERS))
	(cd ..; tar -czvf ski/ski-$(VERS).tar.gz `cat ski/MANIFEST`)
	@(cd ..; rm ski-$(VERS))

dist: ski-$(VERS).tar.gz

release: ski-$(VERS).tar.gz ski.html
	shipper -f; rm -f CHANGES ANNOUNCE* *.6 *.html *.rpm *.lsm

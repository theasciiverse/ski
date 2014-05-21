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

pychecker:
	@ln -f ski ski.py
	@-pychecker --only --limit 50 ski.py
	@rm -f ski.py*

COMMON_PYLINT = --rcfile=/dev/null --reports=n --msg-template="{path}:{line}: [{msg_id}({symbol}), {obj}] {msg}"
PYLINTOPTS = $(COMMON_PYLINT) --disable="C0103,C0111,C0301,C0325,C0326,C1001,R0902,R0911,R0912,W0621"
pylint:
	@pylint $(PYLINTOPTS) ski

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

SHELL:=/bin/bash

.DEFAULT_GOAL := default

%.svg: %.dot
	dot -Tsvg $< -o $@

default: $(patsubst %.dot,%.svg,$(wildcard *.dot))

clean:
	rm
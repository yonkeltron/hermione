SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

ifeq ($(origin .RECIPEPREFIX), undefined)
  $(error This Make does not support .RECIPEPREFIX. Please use GNU Make 4.0 or later)
endif
.RECIPEPREFIX = >

PUBLIC_PATH = public

test:
> cargo test -- --test-threads=1

check:
> cargo clippy

site:
> mkdir -p $(PUBLIC_PATH)
> asciidoctor README.adoc -o $(PUBLIC_PATH)/index.html

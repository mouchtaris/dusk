#!/bin/sh
cargo \
	--color always run \
	--release \
	--bin xs2 \
	-- ci/bootstrap.exe

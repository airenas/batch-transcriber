-include Makefile.options
###############################################################################
## build docker for provided service
docker/%/build: 
	cd build/$* && $(MAKE) dbuild
.PHONY: docker/*/build	
###############################################################################

.EXPORT_ALL_VARIABLES:

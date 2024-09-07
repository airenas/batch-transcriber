include Makefile.options
########################################################################################################
## print usage information
help:
	@echo 'Usage:'
	@cat ${MAKEFILE_LIST} | grep -e "^## " -A 1 | grep -v '\-\-' | sed 's/^##//' | cut -f1 -d":" | \
		awk '{info=$$0; getline; print "  " $$0 ": " info;}' | column -t -s ':' | sort
########################################################################################################
########################################################################################################
install: init/dir .env
	docker compose up -d --scale file-adder=0
.PHONY: install
.env: .env.in 
	cat $< | envsubst > $@
########################################################################################################
status: 
	docker compose ps
.PHONY: status
########################################################################################################
psql:
	docker compose exec -e PGPASSWORD=${postgres_pass} postgres psql -U postgres -h localhost -d asr
.PHONY: psql	
migrate:
	docker compose up dbmigrate
.PHONY: migrate
########################################################################################################
init/dir: | ${data_dir}/incoming ${data_dir}/working ${data_dir}/failed ${data_dir}/processed
.PHONY: init/dir
${data_dir}/incoming ${data_dir}/working ${data_dir}/failed ${data_dir}/processed:
	mkdir -p $@
########################################################################################################
clean:
	docker compose down
.PHONY: clean
########################################################################################################
.EXPORT_ALL_VARIABLES:	

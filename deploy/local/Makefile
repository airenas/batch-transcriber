-include .env
##################################################
start: init/dir
	docker compose up -d
.PHONY: start
psql:
	docker compose exec -e PGPASSWORD=postgres postgres psql -U postgres -h localhost -d asr
migrate:
	docker compose up dbmigrate
init/dir: 
	mkdir -p .data/incoming	
	mkdir -p .data/working	
	mkdir -p .data/failed	
	mkdir -p .data/processed	
##################################################
clean:
	docker compose down
.PHONY: clean
##################################################
.EXPORT_ALL_VARIABLES:	
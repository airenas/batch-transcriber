-include .env
##################################################
start: 
	docker compose up -d
.PHONY: start
psql:
	docker compose exec -e PGPASSWORD=postgres postgres psql -U postgres -h localhost -d asr
migrate:
	docker compose up dbmigrate
##################################################
clean:
	docker compose down
.PHONY: clean
##################################################
.EXPORT_ALL_VARIABLES:	
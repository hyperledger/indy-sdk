docker-build:
	docker-compose build

docker-start:
	docker-compose up
	docker-compose stop
	make docker-clean

docker-exec:
	docker exec -it vcx-demo-libindyvcx bash

docker-clean:
	docker rm -f vcx-demo-libindyvcx vcx-demo-indypool vcx-demo-dummy-cloud-agent
	docker network rm demo_demo_network
	docker volume rm demo_sandbox
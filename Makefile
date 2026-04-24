# apiserver-cache Makefile

BINARY_NAME := apiserver-cache
IMAGE_NAME := apiserver-cache
IMAGE_TAG := latest
DOCKER_REGISTRY ?= 
NAMESPACE ?= default

# Build targets
.PHONY: build build-release run test clean fmt lint

build:
	cargo build

build-release:
	cargo build --release

run:
	cargo run -- --config config.yaml

test:
	cargo test

clean:
	cargo clean

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

# Docker targets
.PHONY: docker-build docker-push docker-run

docker-build:
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) .

docker-push: docker-build
ifeq ($(DOCKER_REGISTRY),)
	@echo "DOCKER_REGISTRY is not set, skipping push"
	@exit 1
else
	docker tag $(IMAGE_NAME):$(IMAGE_TAG) $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
	docker push $(DOCKER_REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)
endif

docker-run:
	docker run -p 8080:8080 $(IMAGE_NAME):$(IMAGE_TAG)

# Kubernetes targets
.PHONY: k8s-deploy k8s-delete k8s-apply k8s-logs k8s-status

k8s-deploy:
	kubectl apply -f deploy/namespace.yaml -n $(NAMESPACE) || true
	kubectl apply -f deploy/configmap.yaml -n $(NAMESPACE)
	kubectl apply -f deploy/deployment.yaml -n $(NAMESPACE)
	kubectl apply -f deploy/service.yaml -n $(NAMESPACE)

k8s-delete:
	kubectl delete -f deploy/deployment.yaml -n $(NAMESPACE) --ignore-not-found=true
	kubectl delete -f deploy/service.yaml -n $(NAMESPACE) --ignore-not-found=true
	kubectl delete -f deploy/configmap.yaml -n $(NAMESPACE) --ignore-not-found=true

k8s-apply: k8s-deploy

k8s-logs:
	kubectl logs -l app=$(BINARY_NAME) -n $(NAMESPACE) --tail=100 -f

k8s-status:
	kubectl get pods -l app=$(BINARY_NAME) -n $(NAMESPACE)
	kubectl get svc -l app=$(BINARY_NAME) -n $(NAMESPACE)

# All-in-one
.PHONY: all

all: build-release docker-build k8s-deploy

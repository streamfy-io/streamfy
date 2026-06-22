ORG=myorg
IMAGE=http-source
VERSION=0.3.8

copy-profile:
	streamfy profile export > streamfy_profile.toml

build: copy-profile
	docker build --progress=plain -t $(ORG)/$(IMAGE):$(VERSION) .

run: build
	docker run --network host --rm  $(ORG)/$(IMAGE):$(VERSION)